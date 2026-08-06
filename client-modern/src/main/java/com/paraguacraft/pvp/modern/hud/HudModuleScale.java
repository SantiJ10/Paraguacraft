package com.paraguacraft.pvp.modern.hud;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.gui.DrawContext;
import org.joml.Matrix3x2fStack;

/** Escala proporcional por módulo (50–200%), alineada a píxel para no emborronar. */
public final class HudModuleScale {

    public static final int MIN = 50;
    public static final int MAX = 200;

    private static int crispDepth = 0;

    private HudModuleScale() {}

    public static boolean isCrisp() {
        return crispDepth > 0;
    }

    public static int clamp(int pct) {
        if (pct < MIN) return MIN;
        if (pct > MAX) return MAX;
        return pct;
    }

    public static float factor(int pct) {
        return clamp(pct) / 100f;
    }

    public static void begin(DrawContext ctx, float x, float y, int scalePct) {
        Matrix3x2fStack m = ctx.getMatrices();
        m.pushMatrix();
        // Origen en píxel entero evita subpíxeles (texto/items nítidos).
        m.translate((float) Math.floor(x), (float) Math.floor(y));
        float s = factor(scalePct);
        if (s != 1f) {
            m.scale(s, s);
        }
        crispDepth++;
    }

    public static void end(DrawContext ctx) {
        if (crispDepth > 0) {
            crispDepth--;
        }
        ctx.getMatrices().popMatrix();
    }

    public static int get(int boxId) {
        return switch (boxId) {
            case 0 -> ModernConfig.scaleFps;
            case 1 -> ModernConfig.scalePing;
            case 2 -> ModernConfig.scaleCps;
            case 3 -> ModernConfig.scaleKeys;
            case 4 -> ModernConfig.scaleArmor;
            case 5 -> ModernConfig.scaleHeld;
            case 6 -> ModernConfig.scaleBwRes;
            case 7 -> ModernConfig.scaleHardware;
            case 8 -> ModernConfig.scaleBlocks;
            case 9 -> ModernConfig.scalePotions;
            case 10 -> ModernConfig.scaleCoords;
            case 11 -> ModernConfig.scaleCompass;
            case 12 -> ModernConfig.scaleCombo;
            case 13 -> ModernConfig.scaleMusic;
            case 14 -> ModernConfig.scaleCombat;
            case 15 -> ModernConfig.scaleGameMode;
            case 16 -> ModernConfig.scaleBridge;
            case 17 -> ModernConfig.scaleReach;
            case 18 -> ModernConfig.scaleServer;
            default -> 100;
        };
    }

    public static void set(int boxId, int pct) {
        pct = clamp(pct);
        switch (boxId) {
            case 0 -> ModernConfig.scaleFps = pct;
            case 1 -> ModernConfig.scalePing = pct;
            case 2 -> ModernConfig.scaleCps = pct;
            case 3 -> ModernConfig.scaleKeys = pct;
            case 4 -> ModernConfig.scaleArmor = pct;
            case 5 -> ModernConfig.scaleHeld = pct;
            case 6 -> ModernConfig.scaleBwRes = pct;
            case 7 -> ModernConfig.scaleHardware = pct;
            case 8 -> ModernConfig.scaleBlocks = pct;
            case 9 -> ModernConfig.scalePotions = pct;
            case 10 -> ModernConfig.scaleCoords = pct;
            case 11 -> ModernConfig.scaleCompass = pct;
            case 12 -> ModernConfig.scaleCombo = pct;
            case 13 -> ModernConfig.scaleMusic = pct;
            case 14 -> ModernConfig.scaleCombat = pct;
            case 15 -> ModernConfig.scaleGameMode = pct;
            case 16 -> ModernConfig.scaleBridge = pct;
            case 17 -> ModernConfig.scaleReach = pct;
            case 18 -> ModernConfig.scaleServer = pct;
            default -> {}
        }
    }
}
