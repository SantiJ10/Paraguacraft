package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;

/** Combo counter PvP (sin reach display). */
public final class CombatStats {

    public static int comboCount;
    private static long lastHitTime;

    private CombatStats() {}

    public static void onAttack() {
        if (!ModernConfig.comboCounter) {
            return;
        }
        long now = System.currentTimeMillis();
        if (now - lastHitTime > 3000L) {
            comboCount = 0;
        }
        comboCount++;
        lastHitTime = now;
    }

    public static void onPlayerHurt() {
        if (ModernConfig.comboCounter) {
            comboCount = 0;
        }
    }

    public static void reset() {
        comboCount = 0;
        lastHitTime = 0L;
    }
}
