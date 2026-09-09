package com.paraguacraft.pvp.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.network.NetworkPlayerInfo;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.Tessellator;
import net.minecraft.client.renderer.WorldRenderer;
import net.minecraft.client.renderer.entity.RenderManager;
import net.minecraft.client.renderer.vertex.DefaultVertexFormats;
import net.minecraft.entity.Entity;
import net.minecraft.entity.item.EntityTNTPrimed;
import net.minecraftforge.client.event.RenderWorldLastEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import org.lwjgl.opengl.GL11;

import java.util.HashMap;
import java.util.HashSet;
import java.util.Iterator;
import java.util.Map;
import java.util.Set;

/**
 * Contador de TNT sincronizado: reloj de pared (no depende del TPS del cliente),
 * compensación de ping y fuse override por servidor (p. ej. 52 ticks en practice).
 */
public class TntCountdownRenderer {

    private static final class Track {
        long seenNanos;
        int startFuse;
    }

    private static final Map<Integer, Track> TRACKS = new HashMap<Integer, Track>();
    private final Minecraft mc = Minecraft.getMinecraft();

    @SubscribeEvent
    public void onRenderWorldLast(RenderWorldLastEvent event) {
        if (!ModConfig.showTntCountdown || mc.theWorld == null || mc.getRenderViewEntity() == null) {
            return;
        }

        float partialTicks = event.partialTicks;
        RenderManager rm = mc.getRenderManager();
        FontRenderer font = rm.getFontRenderer();
        if (font == null) {
            return;
        }

        Entity view = mc.getRenderViewEntity();
        double camX = view.lastTickPosX + (view.posX - view.lastTickPosX) * partialTicks;
        double camY = view.lastTickPosY + (view.posY - view.lastTickPosY) * partialTicks;
        double camZ = view.lastTickPosZ + (view.posZ - view.lastTickPosZ) * partialTicks;

        Set<Integer> seen = new HashSet<Integer>();
        for (Entity entity : mc.theWorld.loadedEntityList) {
            if (!(entity instanceof EntityTNTPrimed)) {
                continue;
            }
            EntityTNTPrimed tnt = (EntityTNTPrimed) entity;
            seen.add(tnt.getEntityId());
            float fuseLeft = remainingTicks(tnt, partialTicks);
            if (fuseLeft <= 0.0F) {
                continue;
            }
            drawLabel(tnt, camX, camY, camZ, partialTicks, rm, font, fuseLeft);
        }
        prune(seen);
    }

    private float remainingTicks(EntityTNTPrimed tnt, float partialTicks) {
        int id = tnt.getEntityId();
        Track track = TRACKS.get(id);
        if (track == null) {
            track = new Track();
            long pingOffset = 0L;
            if (ModConfig.tntPingCompensate) {
                pingOffset = (long) (pingMs() * 500_000.0D);
            }
            track.seenNanos = System.nanoTime() - pingOffset;
            track.startFuse = Math.max(1, tnt.fuse);
            TRACKS.put(id, track);
        }
        int fuseCap = resolveFuse(track.startFuse);
        float elapsed = (System.nanoTime() - track.seenNanos) / 50_000_000.0F;
        return Math.max(0.0F, fuseCap - elapsed);
    }

    private int resolveFuse(int startFuse) {
        if (ModConfig.tntFuseOverride > 0) {
            return ModConfig.tntFuseOverride;
        }
        if (mc.getCurrentServerData() != null && mc.getCurrentServerData().serverIP != null) {
            String ip = mc.getCurrentServerData().serverIP.toLowerCase();
            if (ip.contains("minemen") || ip.contains("mineman") || ip.contains("mmc.") || ip.contains("minemenclub")) {
                return 52;
            }
        }
        return startFuse;
    }

    private float pingMs() {
        if (mc.getNetHandler() == null || mc.thePlayer == null) {
            return 0.0F;
        }
        NetworkPlayerInfo info = mc.getNetHandler().getPlayerInfo(mc.thePlayer.getUniqueID());
        if (info == null) {
            return 0.0F;
        }
        return Math.max(0, info.getResponseTime());
    }

    private static void prune(Set<Integer> alive) {
        Iterator<Map.Entry<Integer, Track>> it = TRACKS.entrySet().iterator();
        while (it.hasNext()) {
            if (!alive.contains(it.next().getKey())) {
                it.remove();
            }
        }
    }

    private static void drawLabel(
        EntityTNTPrimed tnt,
        double camX,
        double camY,
        double camZ,
        float partialTicks,
        RenderManager rm,
        FontRenderer font,
        float fuseLeft
    ) {
        String text = String.format(java.util.Locale.US, "%.2f", fuseLeft / 20.0F).replace('.', ',');
        int rgb = fuseColor(fuseLeft);

        double x = tnt.lastTickPosX + (tnt.posX - tnt.lastTickPosX) * partialTicks - camX;
        double y = tnt.lastTickPosY + (tnt.posY - tnt.lastTickPosY) * partialTicks - camY + tnt.height + 0.45D;
        double z = tnt.lastTickPosZ + (tnt.posZ - tnt.lastTickPosZ) * partialTicks - camZ;

        float scale = 0.026666668F;
        GlStateManager.pushMatrix();
        GlStateManager.translate((float) x, (float) y, (float) z);
        GL11.glNormal3f(0.0F, 1.0F, 0.0F);
        GlStateManager.rotate(-rm.playerViewY, 0.0F, 1.0F, 0.0F);
        GlStateManager.rotate(rm.playerViewX * (Minecraft.getMinecraft().gameSettings.thirdPersonView == 2 ? -1.0F : 1.0F), 1.0F, 0.0F, 0.0F);
        GlStateManager.scale(-scale, -scale, scale);
        GlStateManager.disableLighting();
        GlStateManager.depthMask(false);
        GlStateManager.disableDepth();
        GlStateManager.enableBlend();
        GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
        GlStateManager.disableTexture2D();

        int w = font.getStringWidth(text) / 2;
        Tessellator tess = Tessellator.getInstance();
        WorldRenderer wr = tess.getWorldRenderer();
        wr.begin(7, DefaultVertexFormats.POSITION_COLOR);
        wr.pos(-w - 1, -1.0D, 0.0D).color(0.0F, 0.0F, 0.0F, 0.35F).endVertex();
        wr.pos(-w - 1, 8.0D, 0.0D).color(0.0F, 0.0F, 0.0F, 0.35F).endVertex();
        wr.pos(w + 1, 8.0D, 0.0D).color(0.0F, 0.0F, 0.0F, 0.35F).endVertex();
        wr.pos(w + 1, -1.0D, 0.0D).color(0.0F, 0.0F, 0.0F, 0.35F).endVertex();
        tess.draw();

        GlStateManager.enableTexture2D();
        font.drawString(text, -w, 0, 0x20000000 | (rgb & 0xFFFFFF));
        GlStateManager.enableDepth();
        GlStateManager.depthMask(true);
        font.drawStringWithShadow(text, -w, 0, 0xFF000000 | (rgb & 0xFFFFFF));

        GlStateManager.disableBlend();
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
        GlStateManager.enableLighting();
        GlStateManager.popMatrix();
    }

    private static int fuseColor(float fuseLeft) {
        float sec = fuseLeft / 20.0F;
        if (sec > 2.0F) {
            return 0x55FF55;
        }
        if (sec > 1.0F) {
            return 0xFFFF55;
        }
        return 0xFF5555;
    }
}
