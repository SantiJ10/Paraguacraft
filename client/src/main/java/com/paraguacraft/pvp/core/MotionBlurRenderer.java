package com.paraguacraft.pvp.core;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.Tessellator;
import net.minecraft.client.renderer.WorldRenderer;
import net.minecraft.client.renderer.vertex.DefaultVertexFormats;
import net.minecraft.client.shader.Framebuffer;
import org.lwjgl.opengl.GL11;

/**
 * Motion blur por accumulation buffer (mezcla el frame actual con el anterior).
 * Se aplica al final del mundo, antes del HUD, para no desenfocar la GUI.
 */
public final class MotionBlurRenderer {

    private static Framebuffer accum;
    private static int lastW;
    private static int lastH;
    private static boolean primed;

    private MotionBlurRenderer() {}

    public static void afterWorld() {
        if (!ModConfig.motionBlurEnabled || ModConfig.motionBlurAmount <= 0.001f) {
            primed = false;
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc == null || mc.theWorld == null || mc.currentScreen != null) {
            primed = false;
            return;
        }
        Framebuffer src = mc.getFramebuffer();
        if (src == null) {
            return;
        }
        int w = src.framebufferWidth;
        int h = src.framebufferHeight;
        if (w <= 0 || h <= 0) {
            return;
        }
        ensure(w, h);
        float mix = blendFactor();

        GlStateManager.pushMatrix();
        GlStateManager.matrixMode(GL11.GL_PROJECTION);
        GlStateManager.pushMatrix();
        GlStateManager.loadIdentity();
        GlStateManager.ortho(0.0D, w, h, 0.0D, 1000.0D, 3000.0D);
        GlStateManager.matrixMode(GL11.GL_MODELVIEW);
        GlStateManager.loadIdentity();
        GlStateManager.translate(0.0F, 0.0F, -2000.0F);

        GlStateManager.disableDepth();
        GlStateManager.disableLighting();
        GlStateManager.disableFog();
        GlStateManager.disableAlpha();
        GlStateManager.enableTexture2D();
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);

        if (primed) {
            src.bindFramebuffer(true);
            GlStateManager.enableBlend();
            GlStateManager.tryBlendFuncSeparate(GL11.GL_SRC_ALPHA, GL11.GL_ONE_MINUS_SRC_ALPHA, 1, 0);
            drawTexture(accum.framebufferTexture, w, h, mix);
            GlStateManager.disableBlend();
        }

        accum.bindFramebuffer(true);
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
        drawTexture(src.framebufferTexture, w, h, 1.0F);
        src.bindFramebuffer(true);
        primed = true;

        GlStateManager.enableDepth();
        GlStateManager.enableAlpha();
        GlStateManager.matrixMode(GL11.GL_PROJECTION);
        GlStateManager.popMatrix();
        GlStateManager.matrixMode(GL11.GL_MODELVIEW);
        GlStateManager.popMatrix();
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
    }

    private static float blendFactor() {
        float a = Math.max(0.0F, Math.min(1.0F, ModConfig.motionBlurAmount));
        if (ModConfig.motionBlurType == 1) {
            // V2: curva un poco más agresiva en valores altos.
            return a * a * 0.85F + a * 0.1F;
        }
        return a * 0.85F;
    }

    private static void drawTexture(int tex, int w, int h, float alpha) {
        GlStateManager.bindTexture(tex);
        GlStateManager.color(1.0F, 1.0F, 1.0F, alpha);
        Tessellator tess = Tessellator.getInstance();
        WorldRenderer wr = tess.getWorldRenderer();
        wr.begin(GL11.GL_QUADS, DefaultVertexFormats.POSITION_TEX);
        wr.pos(0.0D, h, 0.0D).tex(0.0D, 0.0D).endVertex();
        wr.pos(w, h, 0.0D).tex(1.0D, 0.0D).endVertex();
        wr.pos(w, 0.0D, 0.0D).tex(1.0D, 1.0D).endVertex();
        wr.pos(0.0D, 0.0D, 0.0D).tex(0.0D, 1.0D).endVertex();
        tess.draw();
    }

    private static void ensure(int w, int h) {
        if (accum != null && lastW == w && lastH == h) {
            return;
        }
        if (accum != null) {
            accum.deleteFramebuffer();
        }
        accum = new Framebuffer(w, h, true);
        lastW = w;
        lastH = h;
        primed = false;
    }
}
