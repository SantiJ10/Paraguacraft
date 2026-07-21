package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.entity.Entity;
import net.minecraft.entity.LivingEntity;

import java.util.HashMap;
import java.util.Iterator;
import java.util.Map;

/**
 * Reach display, combo counter y estadisticas de combate por sesion.
 *
 * <p>Golpes, mejor combo y muertes propias son 100% confiables (cliente-only).
 * "Posibles bajas" es un heuristico: se acredita si un ente golpeado hace poco
 * desaparece del mundo (ver {@link #onEntityUnload}) — sin un plugin de servidor
 * no hay confirmacion real de kill, por eso se muestra siempre como estimado.
 */
public final class CombatStats {

    private static final long RECENT_HIT_WINDOW_MS = 5000L;

    public static double lastReach = 0.0;
    public static int comboCount;
    private static long lastHitTime;

    public static int hits;
    public static int bestCombo;
    public static int deaths;
    public static int possibleKills;

    private static final Map<Integer, Long> recentlyHitEntities = new HashMap<>();

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
            if (comboCount > bestCombo) {
                bestCombo = comboCount;
            }
        }
        if (target instanceof LivingEntity && target != client.player) {
            hits++;
            recentlyHitEntities.put(target.getId(), System.currentTimeMillis());
        }
    }

    public static void onPlayerHurt() {
        if (ModernConfig.comboCounter) {
            comboCount = 0;
        }
    }

    public static void onOwnDeath() {
        deaths++;
    }

    /** Registrar en {@code ClientEntityEvents.ENTITY_UNLOAD}. */
    public static void onEntityUnload(Entity entity) {
        Long hitAt = recentlyHitEntities.remove(entity.getId());
        if (hitAt != null && System.currentTimeMillis() - hitAt <= RECENT_HIT_WINDOW_MS) {
            possibleKills++;
        }
    }

    /** Llamar cada tick: descarta golpes viejos para no acreditar bajas tardias ni acumular memoria. */
    public static void pruneRecentHits() {
        if (recentlyHitEntities.isEmpty()) {
            return;
        }
        long now = System.currentTimeMillis();
        Iterator<Map.Entry<Integer, Long>> it = recentlyHitEntities.entrySet().iterator();
        while (it.hasNext()) {
            if (now - it.next().getValue() > RECENT_HIT_WINDOW_MS) {
                it.remove();
            }
        }
    }

    /** Reset de sesion (join a un servidor/mundo nuevo). */
    public static void reset() {
        lastReach = 0.0;
        comboCount = 0;
        lastHitTime = 0L;
        hits = 0;
        bestCombo = 0;
        deaths = 0;
        possibleKills = 0;
        recentlyHitEntities.clear();
    }
}
