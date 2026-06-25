package com.paraguacraft.pvp.network;

import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

/** Cache client-side de insignias recibidas del servidor. */
public final class BadgeRegistry {

    private static final Map<UUID, Byte> BADGES = new ConcurrentHashMap<UUID, Byte>();

    private BadgeRegistry() {}

    public static void clear() {
        BADGES.clear();
    }

    public static void set(UUID id, byte badge) {
        if (badge == BadgeProtocol.BADGE_NONE) {
            BADGES.remove(id);
        } else {
            BADGES.put(id, badge);
        }
    }

    public static boolean hasBadge(UUID id) {
        return BADGES.containsKey(id);
    }

    public static byte getBadge(UUID id) {
        Byte b = BADGES.get(id);
        return b != null ? b : BadgeProtocol.BADGE_NONE;
    }
}
