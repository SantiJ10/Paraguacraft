package com.paraguacraft.pvp.modern.network;

import net.minecraft.client.MinecraftClient;
import net.minecraft.entity.player.PlayerEntity;

import java.util.Collections;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Simula el backend de usuarios Paraguacraft. El mini-logo se valida contra
 * {@link #onlineUsers}.
 */
public final class ParaguacraftNetwork {

    public static final Set<UUID> onlineUsers =
        Collections.newSetFromMap(new ConcurrentHashMap<UUID, Boolean>());

    static {
        onlineUsers.add(UUID.fromString("00000000-0000-0000-0000-000000000001"));
        onlineUsers.add(UUID.fromString("069a79f4-44e9-4726-a5be-fca90e38aaf5"));
    }

    private ParaguacraftNetwork() {}

    public static void tickLocal() {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.player != null) {
            onlineUsers.add(client.player.getUuid());
        }
    }

    public static boolean hasLogo(UUID id) {
        if (id == null) {
            return false;
        }
        if (onlineUsers.contains(id) || BadgeRegistry.hasBadge(id)) {
            return true;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        return client.player != null && id.equals(client.player.getUuid());
    }

    public static boolean hasLogo(PlayerEntity player) {
        return player != null && hasLogo(player.getUuid());
    }
}
