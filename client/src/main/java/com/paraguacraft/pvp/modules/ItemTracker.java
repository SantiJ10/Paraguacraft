package com.paraguacraft.pvp.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.Tessellator;
import net.minecraft.client.renderer.WorldRenderer;
import net.minecraft.client.renderer.entity.RenderManager;
import net.minecraft.client.renderer.vertex.DefaultVertexFormats;
import net.minecraft.entity.Entity;
import net.minecraft.entity.item.EntityItem;
import net.minecraft.item.ItemStack;
import net.minecraftforge.client.event.RenderWorldLastEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import org.lwjgl.opengl.GL11;

import java.util.ArrayList;
import java.util.Collections;
import java.util.Comparator;
import java.util.List;

/** Rastrea items dropeados: lista 2D compacta y etiquetas 3D discretas. */
public final class ItemTracker {

    private static final double RANGE_2D = 16.0D;
    private static final double RANGE_3D = 10.0D;
    private static final int HUD_MAX = 5;
    private static final int WORLD_MAX = 4;
    private static final float HUD_SCALE = 0.72F;
    private static final int ROW = 10;
    private static final float LABEL_SCALE = 0.016F;

    public ItemTracker() {}

    public static void drawHud() {
        if (!ModConfig.itemTracker2d) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        FontRenderer fr = mc.fontRendererObj;
        if (fr == null) {
            return;
        }
        List<EntityItem> items = nearby(RANGE_2D);
        int n = Math.min(HUD_MAX, items.size());
        GlStateManager.pushMatrix();
        try {
            GlStateManager.scale(HUD_SCALE, HUD_SCALE, 1.0F);
            GlStateManager.enableBlend();
            GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
            int y = 0;
            for (int i = 0; i < n; i++) {
                EntityItem ei = items.get(i);
                ItemStack stack = ei.getEntityItem();
                if (stack == null) {
                    continue;
                }
                int dist = (int) Math.ceil(mc.thePlayer.getDistanceToEntity(ei));
                String line = stack.getDisplayName() + " x" + stack.stackSize + "  " + dist + "m";
                int w = fr.getStringWidth(line);
                drawBg(-2, y - 1, w + 4, ROW, 0.22F);
                fr.drawStringWithShadow(line, 0, y, i == 0 ? 0xFFE8E8E8 : 0xFFAAAAAA);
                y += ROW;
            }
        } finally {
            GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
            GlStateManager.popMatrix();
        }
    }

    public static int hudHeight() {
        if (!ModConfig.itemTracker2d) {
            return 10;
        }
        int n = Math.max(1, Math.min(HUD_MAX, nearby(RANGE_2D).size()));
        return Math.max(10, Math.round(n * ROW * HUD_SCALE) + 2);
    }

    @SubscribeEvent
    public void onWorldLast(RenderWorldLastEvent event) {
        if (!ModConfig.itemTracker3d) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.theWorld == null || mc.getRenderViewEntity() == null) {
            return;
        }
        float partial = event.partialTicks;
        RenderManager rm = mc.getRenderManager();
        FontRenderer font = rm.getFontRenderer();
        if (font == null) {
            return;
        }
        Entity view = mc.getRenderViewEntity();
        double camX = view.lastTickPosX + (view.posX - view.lastTickPosX) * partial;
        double camY = view.lastTickPosY + (view.posY - view.lastTickPosY) * partial;
        double camZ = view.lastTickPosZ + (view.posZ - view.lastTickPosZ) * partial;
        int shown = 0;
        for (EntityItem ei : nearby(RANGE_3D)) {
            if (shown >= WORLD_MAX) {
                break;
            }
            if (!inView(view, ei, 0.55D)) {
                continue;
            }
            ItemStack stack = ei.getEntityItem();
            if (stack == null) {
                continue;
            }
            double x = ei.lastTickPosX + (ei.posX - ei.lastTickPosX) * partial - camX;
            double y = ei.lastTickPosY + (ei.posY - ei.lastTickPosY) * partial - camY + 0.42D;
            double z = ei.lastTickPosZ + (ei.posZ - ei.lastTickPosZ) * partial - camZ;
            String label = stack.stackSize > 1
                ? stack.getDisplayName() + " x" + stack.stackSize
                : stack.getDisplayName();
            double dist = mc.thePlayer.getDistanceToEntity(ei);
            float alpha = (float) Math.max(0.35D, 1.0D - dist / RANGE_3D);
            drawLabel(x, y, z, label, rm, font, alpha);
            shown++;
        }
    }

    private static boolean inView(Entity view, EntityItem item, double minDot) {
        double dx = item.posX - view.posX;
        double dy = item.posY - (view.posY + view.getEyeHeight());
        double dz = item.posZ - view.posZ;
        double len = Math.sqrt(dx * dx + dy * dy + dz * dz);
        if (len < 0.001D) {
            return true;
        }
        dx /= len;
        dy /= len;
        dz /= len;
        float yaw = view.rotationYaw;
        float pitch = view.rotationPitch;
        double yawRad = Math.toRadians(yaw);
        double pitchRad = Math.toRadians(pitch);
        double lx = -Math.sin(yawRad) * Math.cos(pitchRad);
        double ly = -Math.sin(pitchRad);
        double lz = Math.cos(yawRad) * Math.cos(pitchRad);
        return dx * lx + dy * ly + dz * lz >= minDot;
    }

    private static void drawLabel(
        double x, double y, double z, String text, RenderManager rm, FontRenderer font, float alpha
    ) {
        GlStateManager.pushMatrix();
        try {
            GlStateManager.translate((float) x, (float) y, (float) z);
            GL11.glNormal3f(0.0F, 1.0F, 0.0F);
            GlStateManager.rotate(-rm.playerViewY, 0.0F, 1.0F, 0.0F);
            float pitchSign = Minecraft.getMinecraft().gameSettings.thirdPersonView == 2 ? -1.0F : 1.0F;
            GlStateManager.rotate(rm.playerViewX * pitchSign, 1.0F, 0.0F, 0.0F);
            GlStateManager.scale(-LABEL_SCALE, -LABEL_SCALE, LABEL_SCALE);
            GlStateManager.disableLighting();
            GlStateManager.depthMask(false);
            GlStateManager.enableBlend();
            GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
            GlStateManager.disableTexture2D();
            int w = font.getStringWidth(text) / 2;
            Tessellator tess = Tessellator.getInstance();
            WorldRenderer wr = tess.getWorldRenderer();
            wr.begin(7, DefaultVertexFormats.POSITION_COLOR);
            wr.pos(-w - 1, -1.0D, 0.0D).color(0.0F, 0.0F, 0.0F, 0.18F * alpha).endVertex();
            wr.pos(-w - 1, 8.0D, 0.0D).color(0.0F, 0.0F, 0.0F, 0.18F * alpha).endVertex();
            wr.pos(w + 1, 8.0D, 0.0D).color(0.0F, 0.0F, 0.0F, 0.18F * alpha).endVertex();
            wr.pos(w + 1, -1.0D, 0.0D).color(0.0F, 0.0F, 0.0F, 0.18F * alpha).endVertex();
            tess.draw();
            GlStateManager.enableTexture2D();
            int color = ((int) (alpha * 230.0F) << 24) | 0x00DDDDDD;
            font.drawStringWithShadow(text, -w, 0, color);
        } finally {
            GlStateManager.enableDepth();
            GlStateManager.depthMask(true);
            GlStateManager.disableBlend();
            GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
            GlStateManager.enableLighting();
            GlStateManager.popMatrix();
        }
    }

    private static void drawBg(int x, int y, int w, int h, float a) {
        Tessellator tess = Tessellator.getInstance();
        WorldRenderer wr = tess.getWorldRenderer();
        GlStateManager.disableTexture2D();
        wr.begin(7, DefaultVertexFormats.POSITION_COLOR);
        wr.pos(x, y + h, 0.0D).color(0.0F, 0.0F, 0.0F, a).endVertex();
        wr.pos(x + w, y + h, 0.0D).color(0.0F, 0.0F, 0.0F, a).endVertex();
        wr.pos(x + w, y, 0.0D).color(0.0F, 0.0F, 0.0F, a).endVertex();
        wr.pos(x, y, 0.0D).color(0.0F, 0.0F, 0.0F, a).endVertex();
        tess.draw();
        GlStateManager.enableTexture2D();
    }

    private static List<EntityItem> nearby(double range) {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.theWorld == null || mc.thePlayer == null) {
            return Collections.emptyList();
        }
        List<EntityItem> out = new ArrayList<EntityItem>();
        for (Entity e : mc.theWorld.loadedEntityList) {
            if (e instanceof EntityItem && mc.thePlayer.getDistanceToEntity(e) <= range) {
                out.add((EntityItem) e);
            }
        }
        Collections.sort(out, new Comparator<EntityItem>() {
            @Override
            public int compare(EntityItem a, EntityItem b) {
                return Double.compare(
                    mc.thePlayer.getDistanceSqToEntity(a),
                    mc.thePlayer.getDistanceSqToEntity(b)
                );
            }
        });
        return out;
    }
}
