package com.paraguacraft.pvp.cosmetics;

import com.paraguacraft.pvp.network.ParaguacraftNetwork;
import java.util.Set;
import java.util.UUID;
import net.minecraft.entity.player.EntityPlayer;

/** Compat: el set canónico es {@link ParaguacraftNetwork#onlineUsers}. */
public final class ParaguacraftRoster {

    private ParaguacraftRoster() {}

    public static void tickLocal() {
        ParaguacraftNetwork.tickLocal();
    }

    public static boolean hasLogo(UUID id) {
        return ParaguacraftNetwork.hasLogo(id);
    }

    public static boolean hasLogo(EntityPlayer player) {
        return ParaguacraftNetwork.hasLogo(player);
    }

    public static Set<UUID> snapshot() {
        return ParaguacraftNetwork.onlineUsers;
    }
}
