package com.paraguacraft.pvp.core;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.OpenGlHelper;
import net.minecraft.client.renderer.Tessellator;
import net.minecraft.client.renderer.WorldRenderer;
import net.minecraft.client.renderer.vertex.DefaultVertexFormats;
import net.minecraft.client.shader.Framebuffer;
import org.lwjgl.opengl.GL11;
import org.lwjgl.opengl.GL12;

import java.nio.ByteBuffer;

/**
 * Motion blur por accumulation: copia el color buffer actual (FBO o Fast Render)
 * y lo mezcla al frame siguiente. No usa un Framebuffer de Minecraft — con
 * OptiFine Fast Render ese objeto queda en blanco y tapaba el mundo.
 */
public final class MotionBlurRenderer {

    private static int tex;
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
        int w = mc.displayWidth;
        int h = mc.displayHeight;
        Framebuffer fb = mc.getFramebuffer();
        if (fb != null && fb.framebufferWidth > 0 && fb.framebufferHeight > 0) {
            w = fb.framebufferWidth;
            h = fb.framebufferHeight;
        }
        if (w <= 0 || h <= 0) {
            return;
        }

        ensure(w, h);
        float mix = blendFactor();

        int prevTex = GL11.glGetInteger(GL11.GL_TEXTURE_BINDING_2D);
        boolean blend = GL11.glIsEnabled(GL11.GL_BLEND);
        boolean depth = GL11.glIsEnabled(GL11.GL_DEPTH_TEST);
        boolean alpha = GL11.glIsEnabled(GL11.GL_ALPHA_TEST);
        boolean lighting = GL11.glIsEnabled(GL11.GL_LIGHTING);
        boolean fog = GL11.glIsEnabled(GL11.GL_FOG);
        boolean cull = GL11.glIsEnabled(GL11.GL_CULL_FACE);

        GlStateManager.setActiveTexture(OpenGlHelper.lightmapTexUnit);
        GlStateManager.disableTexture2D();
        GlStateManager.setActiveTexture(OpenGlHelper.defaultTexUnit);
        GlStateManager.enableTexture2D();
        GL11.glTexEnvi(GL11.GL_TEXTURE_ENV, GL11.GL_TEXTURE_ENV_MODE, GL11.GL_MODULATE);

        GlStateManager.matrixMode(GL11.GL_PROJECTION);
        GlStateManager.pushMatrix();
        GlStateManager.loadIdentity();
        GlStateManager.ortho(0.0D, w, h, 0.0D, 1000.0D, 3000.0D);
        GlStateManager.matrixMode(GL11.GL_MODELVIEW);
        GlStateManager.pushMatrix();
        GlStateManager.loadIdentity();
        GlStateManager.translate(0.0F, 0.0F, -2000.0F);

        GlStateManager.disableDepth();
        GlStateManager.depthMask(false);
        GlStateManager.disableLighting();
        GlStateManager.disableFog();
        GlStateManager.disableAlpha();
        GlStateManager.disableCull();
        GlStateManager.colorMask(true, true, true, true);

        if (primed) {
            GlStateManager.enableBlend();
            GlStateManager.tryBlendFuncSeparate(GL11.GL_SRC_ALPHA, GL11.GL_ONE_MINUS_SRC_ALPHA, 1, 0);
            draw(w, h, mix);
            GlStateManager.disableBlend();
        }

        GlStateManager.bindTexture(tex);
        GL11.glCopyTexSubImage2D(GL11.GL_TEXTURE_2D, 0, 0, 0, 0, 0, w, h);
        primed = true;

        GlStateManager.depthMask(true);
        if (depth) {
            GlStateManager.enableDepth();
        }
        if (alpha) {
            GlStateManager.enableAlpha();
        }
        if (lighting) {
            GlStateManager.enableLighting();
        }
        if (fog) {
            GlStateManager.enableFog();
        }
        if (cull) {
            GlStateManager.enableCull();
        }
        if (blend) {
            GlStateManager.enableBlend();
        } else {
            GlStateManager.disableBlend();
        }

        GlStateManager.matrixMode(GL11.GL_PROJECTION);
        GlStateManager.popMatrix();
        GlStateManager.matrixMode(GL11.GL_MODELVIEW);
        GlStateManager.popMatrix();
        GlStateManager.bindTexture(prevTex);
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
    }

    private static float blendFactor() {
        float a = Math.max(0.0F, Math.min(1.0F, ModConfig.motionBlurAmount));
        if (ModConfig.motionBlurType == 1) {
            return a * a * 0.85F + a * 0.1F;
        }
        return a * 0.85F;
    }

    private static void draw(int w, int h, float alpha) {
        GlStateManager.bindTexture(tex);
        Tessellator tess = Tessellator.getInstance();
        WorldRenderer wr = tess.getWorldRenderer();
        wr.begin(GL11.GL_QUADS, DefaultVertexFormats.POSITION_TEX_COLOR);
        wr.pos(0.0D, h, 0.0D).tex(0.0D, 0.0D).color(1.0F, 1.0F, 1.0F, alpha).endVertex();
        wr.pos(w, h, 0.0D).tex(1.0D, 0.0D).color(1.0F, 1.0F, 1.0F, alpha).endVertex();
        wr.pos(w, 0.0D, 0.0D).tex(1.0D, 1.0D).color(1.0F, 1.0F, 1.0F, alpha).endVertex();
        wr.pos(0.0D, 0.0D, 0.0D).tex(0.0D, 1.0D).color(1.0F, 1.0F, 1.0F, alpha).endVertex();
        tess.draw();
    }

    private static void ensure(int w, int h) {
        if (tex != 0 && lastW == w && lastH == h) {
            return;
        }
        if (tex != 0) {
            GL11.glDeleteTextures(tex);
        }
        tex = GL11.glGenTextures();
        GlStateManager.bindTexture(tex);
        GL11.glTexParameteri(GL11.GL_TEXTURE_2D, GL11.GL_TEXTURE_MIN_FILTER, GL11.GL_LINEAR);
        GL11.glTexParameteri(GL11.GL_TEXTURE_2D, GL11.GL_TEXTURE_MAG_FILTER, GL11.GL_LINEAR);
        GL11.glTexParameteri(GL11.GL_TEXTURE_2D, GL11.GL_TEXTURE_WRAP_S, GL12.GL_CLAMP_TO_EDGE);
        GL11.glTexParameteri(GL11.GL_TEXTURE_2D, GL11.GL_TEXTURE_WRAP_T, GL12.GL_CLAMP_TO_EDGE);
        GL11.glTexImage2D(GL11.GL_TEXTURE_2D, 0, GL11.GL_RGB8, w, h, 0, GL11.GL_RGB, GL11.GL_UNSIGNED_BYTE, (ByteBuffer) null);
        lastW = w;
        lastH = h;
        primed = false;
    }
}
