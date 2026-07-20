package com.paraguacraft.badges.paper;

import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

/** Registro en memoria de insignias por jugador conectado. */
public final class BadgeRegistry {

    private final Map<UUID, Byte> badges = new ConcurrentHashMap<>();

    public void set(UUID id, byte badge) {
        if (badge == BadgeProtocol.BADGE_NONE) {
            badges.remove(id);
        } else {
            badges.put(id, badge);
        }
    }

    public void remove(UUID id) {
        badges.remove(id);
    }

    public byte get(UUID id) {
        Byte b = badges.get(id);
        return b != null ? b : BadgeProtocol.BADGE_NONE;
    }

    public boolean has(UUID id) {
        return badges.containsKey(id);
    }

    public Map<UUID, Byte> snapshot() {
        return Map.copyOf(badges);
    }

    public void clear() {
        badges.clear();
    }
}
