package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.entity.Entity;

/** Reach display y combo counter para entrenamiento PvP. */
public final class CombatStats {

    public static double lastReach = 0.0;
    public static int comboCount;
    private static long lastHitTime;

    private CombatStats() {}

    public static void onAttack(Entity target) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.player == null || target == null) {
            return;
        }
        if (ModernConfig.reachDisplay) {
            double dx = target.getX() - client.player.getX();
            double dy = target.getEyeY() - client.player.getEyeY();
            double dz = target.getZ() - client.player.getZ();
            lastReach = Math.sqrt(dx * dx + dy * dy + dz * dz);
        }
        if (ModernConfig.comboCounter) {
            long now = System.currentTimeMillis();
            if (now - lastHitTime > 3000L) {
                comboCount = 0;
            }
            comboCount++;
            lastHitTime = now;
        }
    }

    public static void onPlayerHurt() {
        if (ModernConfig.comboCounter) {
            comboCount = 0;
        }
    }

    public static void reset() {
        lastReach = 0.0;
        comboCount = 0;
        lastHitTime = 0L;
    }
}
