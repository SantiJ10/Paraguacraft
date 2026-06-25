package com.paraguacraft.pvp.font;

import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.util.ResourceLocation;
import org.lwjgl.opengl.GL11;

/**
 * Renderizador TTF anti-aliased para UI premium (estilo Lunar / Badlion).
 */
public class CustomFontRenderer {

    private final int textureId;
    private final GlyphPage page;
    private final float scale;

    public CustomFontRenderer(GlyphPage page, float scale) {
        this.page = page;
        this.scale = scale;
        this.textureId = page.getTexture().getGlTextureId();
    }

    public int getFontHeight() {
        return Math.round(page.getFontHeight() * scale);
    }

    public int getStringWidth(String text) {
        if (text == null) {
            return 0;
        }
        int w = 0;
        for (int i = 0; i < text.length(); i++) {
            w += Math.round(page.glyph(text.charAt(i)).xAdvance * scale);
        }
        return w;
    }

    public void drawString(String text, float x, float y, int color) {
        drawString(text, x, y, color, false);
    }

    public void drawCenteredString(String text, float x, float y, int color) {
        drawString(text, x - getStringWidth(text) / 2f, y, color, false);
    }

    public void drawStringWithShadow(String text, float x, float y, int color) {
        drawString(text, x + 1, y + 1, (color & 0xFCFCFC) >> 2 | (color & 0xFF000000), false);
        drawString(text, x, y, color, false);
    }

    public void drawCenteredStringWithShadow(String text, float x, float y, int color) {
        drawStringWithShadow(text, x - getStringWidth(text) / 2f, y, color);
    }

    private void drawString(String text, float x, float y, int color, boolean unused) {
        if (text == null || text.isEmpty()) {
            return;
        }
        bind();

        float a = (float) (color >> 24 & 255) / 255.0F;
        float r = (float) (color >> 16 & 255) / 255.0F;
        float g = (float) (color >> 8 & 255) / 255.0F;
        float b = (float) (color & 255) / 255.0F;
        if (a == 0) {
            a = 1f;
        }

        GlStateManager.enableBlend();
        GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
        GlStateManager.color(r, g, b, a);

        GlStateManager.pushMatrix();
        GlStateManager.scale(scale, scale, 1f);
        float drawX = x / scale;
        float drawY = y / scale;

        GL11.glBegin(GL11.GL_QUADS);
        for (int i = 0; i < text.length(); i++) {
            char c = text.charAt(i);
            GlyphPage.Glyph gl = page.glyph(c);
            float x1 = drawX;
            float y1 = drawY;
            float x2 = drawX + gl.width;
            float y2 = drawY + gl.height;
            GL11.glTexCoord2f(gl.u, gl.v);
            GL11.glVertex2f(x1, y1);
            GL11.glTexCoord2f(gl.u2, gl.v);
            GL11.glVertex2f(x2, y1);
            GL11.glTexCoord2f(gl.u2, gl.v2);
            GL11.glVertex2f(x2, y2);
            GL11.glTexCoord2f(gl.u, gl.v2);
            GL11.glVertex2f(x1, y2);
            drawX += gl.xAdvance;
        }
        GL11.glEnd();

        GlStateManager.popMatrix();
        GlStateManager.color(1f, 1f, 1f, 1f);
        GlStateManager.disableBlend();
    }

    private void bind() {
        GlStateManager.bindTexture(this.textureId);
    }
}
