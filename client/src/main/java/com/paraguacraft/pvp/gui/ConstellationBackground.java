package com.paraguacraft.pvp.gui;

import net.minecraft.client.renderer.GlStateManager;
import org.lwjgl.opengl.GL11;

import java.util.Random;

/** Fondo constelación — nodos + GL_LINES por distancia. */
public final class ConstellationBackground {

    private static final int N = 52;
    private static final float LINK = 88f;
    private static final float LINK_SQ = LINK * LINK;
    private static final Random RNG = new Random(0x504756L);

    private static float[] px;
    private static float[] py;
    private static float[] vx;
    private static float[] vy;
    private static int lastW;
    private static int lastH;

    private ConstellationBackground() {}

    public static void draw(int w, int h, float partialTicks) {
        ensure(w, h);
        step(w, h);

        GlStateManager.disableTexture2D();
        GlStateManager.enableBlend();
        GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
        GlStateManager.disableDepth();

        GL11.glLineWidth(1f);
        GL11.glBegin(GL11.GL_LINES);
        GL11.glColor4f(0f, 0.88f, 1f, 0.32f);
        for (int i = 0; i < N; i++) {
            for (int j = i + 1; j < N; j++) {
                float dx = px[i] - px[j];
                float dy = py[i] - py[j];
                if (dx * dx + dy * dy < LINK_SQ) {
                    GL11.glVertex2f(px[i], py[i]);
                    GL11.glVertex2f(px[j], py[j]);
                }
            }
        }
        GL11.glEnd();

        GL11.glPointSize(2.2f);
        GL11.glBegin(GL11.GL_POINTS);
        GL11.glColor4f(0f, 0.9f, 1f, 0.85f);
        for (int i = 0; i < N; i++) {
            GL11.glVertex2f(px[i], py[i]);
        }
        GL11.glEnd();

        GlStateManager.enableTexture2D();
        GlStateManager.disableBlend();
        GlStateManager.enableDepth();
        GlStateManager.color(1f, 1f, 1f, 1f);
    }

    private static void ensure(int w, int h) {
        if (px != null && w == lastW && h == lastH) {
            return;
        }
        lastW = w;
        lastH = h;
        px = new float[N];
        py = new float[N];
        vx = new float[N];
        vy = new float[N];
        for (int i = 0; i < N; i++) {
            px[i] = RNG.nextFloat() * w;
            py[i] = RNG.nextFloat() * h;
            vx[i] = (RNG.nextFloat() - 0.5f) * 0.35f;
            vy[i] = (RNG.nextFloat() - 0.5f) * 0.35f;
        }
    }

    private static void step(int w, int h) {
        for (int i = 0; i < N; i++) {
            px[i] += vx[i];
            py[i] += vy[i];
            if (px[i] < 0f) {
                px[i] = 0f;
                vx[i] = Math.abs(vx[i]);
            } else if (px[i] > w) {
                px[i] = w;
                vx[i] = -Math.abs(vx[i]);
            }
            if (py[i] < 0f) {
                py[i] = 0f;
                vy[i] = Math.abs(vy[i]);
            } else if (py[i] > h) {
                py[i] = h;
                vy[i] = -Math.abs(vy[i]);
            }
        }
    }
}
