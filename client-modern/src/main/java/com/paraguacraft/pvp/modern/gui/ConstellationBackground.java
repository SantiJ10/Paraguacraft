package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.DrawContext;

import java.util.Random;

/** Constelación animada — nodos conectados (estilo 1.8.9). */
public final class ConstellationBackground {

    private static final int N = 48;
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

    public static void draw(DrawContext ctx, int w, int h) {
        ensure(w, h);
        step(w, h);
        int lineColor = (UiTheme.accent() & 0x00FFFFFF) | 0x52000000;
        int nodeColor = (UiTheme.accent() & 0x00FFFFFF) | 0xD8000000;
        for (int i = 0; i < N; i++) {
            for (int j = i + 1; j < N; j++) {
                float dx = px[i] - px[j];
                float dy = py[i] - py[j];
                if (dx * dx + dy * dy < LINK_SQ) {
                    drawLine(ctx, (int) px[i], (int) py[i], (int) px[j], (int) py[j], lineColor);
                }
            }
        }
        for (int i = 0; i < N; i++) {
            ctx.fill((int) px[i], (int) py[i], (int) px[i] + 2, (int) py[i] + 2, nodeColor);
        }
    }

    private static void drawLine(DrawContext ctx, int x1, int y1, int x2, int y2, int color) {
        int steps = Math.max(Math.abs(x2 - x1), Math.abs(y2 - y1));
        if (steps <= 0) {
            return;
        }
        for (int i = 0; i <= steps; i++) {
            int x = x1 + (x2 - x1) * i / steps;
            int y = y1 + (y2 - y1) * i / steps;
            ctx.fill(x, y, x + 1, y + 1, color);
        }
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
