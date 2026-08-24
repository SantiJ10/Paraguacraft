package com.paraguacraft.pvp.network;

import java.util.Collections;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import net.minecraft.client.Minecraft;
import net.minecraft.entity.player.EntityPlayer;

/**
 * Simula la red de usuarios Paraguacraft. El logo del nametag se valida
 * contra {@link #onlineUsers}.
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
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer != null) {
            onlineUsers.add(mc.thePlayer.getUniqueID());
        }
    }

    public static boolean hasLogo(UUID id) {
        if (id == null) {
            return false;
        }
        if (onlineUsers.contains(id) || BadgeRegistry.hasBadge(id)) {
            return true;
        }
        Minecraft mc = Minecraft.getMinecraft();
        return mc.thePlayer != null && id.equals(mc.thePlayer.getUniqueID());
    }

    public static boolean hasLogo(EntityPlayer player) {
        return player != null && hasLogo(player.getUniqueID());
    }
}
