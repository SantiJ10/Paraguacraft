package com.paraguacraft.pvp.modern.network;

import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

/** Cache client-side de insignias recibidas del servidor (paridad con el cliente 1.8.9). */
public final class BadgeRegistry {

    private static final Map<UUID, Byte> BADGES = new ConcurrentHashMap<>();

    private BadgeRegistry() {}

    public static void clear() {
        BADGES.clear();
    }

    public static void set(UUID id, byte badge) {
        if (id == null) {
            return;
        }
        if (badge == BadgeProtocol.BADGE_NONE) {
            BADGES.remove(id);
        } else {
            BADGES.put(id, badge);
        }
    }

    public static boolean hasBadge(UUID id) {
        return id != null && BADGES.containsKey(id);
    }

    public static byte getBadge(UUID id) {
        if (id == null) {
            return BadgeProtocol.BADGE_NONE;
        }
        Byte b = BADGES.get(id);
        return b != null ? b : BadgeProtocol.BADGE_NONE;
    }
}
