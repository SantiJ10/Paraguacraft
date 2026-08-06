package com.paraguacraft.pvp.hud;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.renderer.GlStateManager;

/** Escala proporcional por módulo (top-left fijo al dibujar). */
public final class HudModuleScale {

    public static final int MIN = 50;
    public static final int MAX = 200;

    private HudModuleScale() {}

    public static int clamp(int pct) {
        if (pct < MIN) {
            return MIN;
        }
        if (pct > MAX) {
            return MAX;
        }
        return pct;
    }

    public static float factor(int pct) {
        return clamp(pct) / 100f;
    }

    /** Empuja GL: origen en (x,y) y escala uniforme. Dibuja en coords locales 0,0+. */
    public static void begin(float x, float y, int scalePct) {
        GlStateManager.pushMatrix();
        GlStateManager.translate(x, y, 0f);
        float s = factor(scalePct);
        if (s != 1f) {
            GlStateManager.scale(s, s, 1f);
        }
    }

    public static void end() {
        GlStateManager.popMatrix();
    }

    public static int scaledSize(int base, int scalePct) {
        return Math.max(1, Math.round(base * factor(scalePct)));
    }

    /** Escala por id de caja del editor (misma nomenclatura que GuiEditHUD). */
    public static int get(int boxId) {
        switch (boxId) {
            case 0: return ModConfig.scaleFps;
            case 1: return ModConfig.scalePing;
            case 2: return ModConfig.scaleCps;
            case 3: return ModConfig.scaleKeys;
            case 4: return ModConfig.scaleArmor;
            case 5: return ModConfig.scalePotions;
            case 6: return ModConfig.scaleCoords;
            case 7: return ModConfig.scaleHeld;
            case 8: return ModConfig.scaleServer;
            case 9: return ModConfig.scaleCompass;
            case 10: return ModConfig.scaleOverlay;
            case 11: return ModConfig.scaleBwRes;
            case 12: return ModConfig.scaleReach;
            case 13: return ModConfig.scaleCombo;
            case 14: return ModConfig.scaleBlocks;
            default: return 100;
        }
    }

    public static void set(int boxId, int pct) {
        pct = clamp(pct);
        switch (boxId) {
            case 0: ModConfig.scaleFps = pct; break;
            case 1: ModConfig.scalePing = pct; break;
            case 2: ModConfig.scaleCps = pct; break;
            case 3: ModConfig.scaleKeys = pct; break;
            case 4: ModConfig.scaleArmor = pct; break;
            case 5: ModConfig.scalePotions = pct; break;
            case 6: ModConfig.scaleCoords = pct; break;
            case 7: ModConfig.scaleHeld = pct; break;
            case 8: ModConfig.scaleServer = pct; break;
            case 9: ModConfig.scaleCompass = pct; break;
            case 10: ModConfig.scaleOverlay = pct; break;
            case 11: ModConfig.scaleBwRes = pct; break;
            case 12: ModConfig.scaleReach = pct; break;
            case 13: ModConfig.scaleCombo = pct; break;
            case 14: ModConfig.scaleBlocks = pct; break;
            default: break;
        }
    }
}
