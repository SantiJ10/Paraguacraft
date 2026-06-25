package com.paraguacraft.pvp.font;

import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.texture.DynamicTexture;
import org.lwjgl.opengl.GL11;

import java.awt.Color;
import java.awt.Font;
import java.awt.FontMetrics;
import java.awt.Graphics2D;
import java.awt.RenderingHints;
import java.awt.image.BufferedImage;

/**
 * Atlas de glifos ASCII (32–126) rasterizado con anti-aliasing AWT → textura OpenGL.
 */
final class GlyphPage {

    static final class Glyph {
        float u, v, u2, v2;
        int width, height;
        int xAdvance;
    }

    private static final int FIRST = 32;
    private static final int COUNT = 96;
    private static final int ATLAS = 512;

    private final Glyph[] glyphs = new Glyph[COUNT];
    private final DynamicTexture texture;
    private final int fontHeight;

    GlyphPage(Font awtFont) {
        BufferedImage img = new BufferedImage(ATLAS, ATLAS, BufferedImage.TYPE_INT_ARGB);
        Graphics2D g = img.createGraphics();
        g.setFont(awtFont);
        g.setColor(Color.WHITE);
        g.setRenderingHint(RenderingHints.KEY_TEXT_ANTIALIASING, RenderingHints.VALUE_TEXT_ANTIALIAS_ON);
        g.setRenderingHint(RenderingHints.KEY_FRACTIONALMETRICS, RenderingHints.VALUE_FRACTIONALMETRICS_ON);
        g.setRenderingHint(RenderingHints.KEY_RENDERING, RenderingHints.VALUE_RENDER_QUALITY);

        FontMetrics fm = g.getFontMetrics();
        this.fontHeight = fm.getAscent();

        int x = 2;
        int y = 2;
        int rowH = 0;

        for (int i = 0; i < COUNT; i++) {
            char c = (char) (FIRST + i);
            int w = Math.max(1, fm.charWidth(c));
            int h = fm.getHeight();
            if (x + w + 2 >= ATLAS) {
                x = 2;
                y += rowH + 2;
                rowH = 0;
            }
            Glyph gl = new Glyph();
            gl.width = w;
            gl.height = h;
            gl.xAdvance = w + 1;
            gl.u = (float) x / ATLAS;
            gl.v = (float) y / ATLAS;
            g.drawString(String.valueOf(c), x, y + fm.getAscent());
            gl.u2 = (float) (x + w) / ATLAS;
            gl.v2 = (float) (y + h) / ATLAS;
            glyphs[i] = gl;
            x += w + 2;
            rowH = Math.max(rowH, h);
        }
        g.dispose();
        this.texture = new DynamicTexture(img);
    }

    int getFontHeight() {
        return fontHeight;
    }

    DynamicTexture getTexture() {
        return texture;
    }

    Glyph glyph(char c) {
        if (c < FIRST || c > 126) {
            return glyphs['?' - FIRST];
        }
        return glyphs[c - FIRST];
    }
}
