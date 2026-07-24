package com.paraguacraft.pvp.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.FontRenderer;
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

/**
 * Contador estilo Lunar sobre TNT encendida.
 * Usa RenderWorldLastEvent para no depender del renderer de entidad (OptiFine lo reemplaza).
 */
public class TntCountdownRenderer {

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

        for (Entity entity : mc.theWorld.loadedEntityList) {
            if (!(entity instanceof EntityTNTPrimed)) {
                continue;
            }
            EntityTNTPrimed tnt = (EntityTNTPrimed) entity;
            if (tnt.fuse <= 0) {
                continue;
            }
            drawLabel(tnt, camX, camY, camZ, partialTicks, rm, font);
        }
    }

    private static void drawLabel(
        EntityTNTPrimed tnt,
        double camX,
        double camY,
        double camZ,
        float partialTicks,
        RenderManager rm,
        FontRenderer font
    ) {
        float fuseLeft = tnt.fuse - partialTicks;
        if (fuseLeft < 0.0F) {
            fuseLeft = 0.0F;
        }
        String text = String.format(java.util.Locale.US, "%.2f", fuseLeft / 20.0F).replace('.', ',');

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
        wr.pos(-w - 1, -1.0D, 0.0D).color(0.0F, 0.0F, 0.0F, 0.25F).endVertex();
        wr.pos(-w - 1, 8.0D, 0.0D).color(0.0F, 0.0F, 0.0F, 0.25F).endVertex();
        wr.pos(w + 1, 8.0D, 0.0D).color(0.0F, 0.0F, 0.0F, 0.25F).endVertex();
        wr.pos(w + 1, -1.0D, 0.0D).color(0.0F, 0.0F, 0.0F, 0.25F).endVertex();
        tess.draw();

        GlStateManager.enableTexture2D();
        font.drawString(text, -w, 0, 0x20FFFFFF);
        GlStateManager.enableDepth();
        GlStateManager.depthMask(true);
        font.drawStringWithShadow(text, -w, 0, 0xFFFFFFFF);

        GlStateManager.disableBlend();
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
        GlStateManager.enableLighting();
        GlStateManager.popMatrix();
    }
}
