package com.paraguacraft.badges;

import org.bukkit.Bukkit;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.event.player.PlayerQuitEvent;
import org.bukkit.plugin.messaging.PluginMessageListener;

import java.io.ByteArrayInputStream;
import java.io.DataInputStream;
import java.util.UUID;

public final class BadgeListener implements Listener, PluginMessageListener {

    private final BadgeService badges;

    BadgeListener(BadgeService badges) {
        this.badges = badges;
    }

    @EventHandler
    public void onJoin(PlayerJoinEvent event) {
        Player player = event.getPlayer();
        if (player.hasPermission("paraguacraft.badge.staff")) {
            UUID id = player.getUniqueId();
            badges.setBadge(id, BadgeProtocol.BADGE_STAFF);
            badges.broadcastUpdate(id, BadgeProtocol.BADGE_STAFF);
        }
        Bukkit.getScheduler().runTaskLater(ParaguacraftBadgesPlugin.getInstance(), new Runnable() {
            @Override
            public void run() {
                badges.sendFullSync(player);
            }
        }, 20L);
    }

    @EventHandler
    public void onQuit(PlayerQuitEvent event) {
        UUID id = event.getPlayer().getUniqueId();
        if (badges.getBadge(id) != BadgeProtocol.BADGE_NONE) {
            badges.remove(id);
            badges.broadcastUpdate(id, BadgeProtocol.BADGE_NONE);
        }
    }

    @Override
    public void onPluginMessageReceived(String channel, Player player, byte[] message) {
        if (!BadgeProtocol.CHANNEL.equals(channel) || message == null || message.length == 0) {
            return;
        }
        try {
            DataInputStream in = new DataInputStream(new ByteArrayInputStream(message));
            byte type = in.readByte();
            if (type != BadgeProtocol.C2S_REGISTER) {
                return;
            }
            byte requested = in.readByte();
            byte badge = BadgeProtocol.BADGE_PARAGUACRAFT;
            if (player.hasPermission("paraguacraft.badge.staff")) {
                badge = BadgeProtocol.BADGE_STAFF;
            } else if (requested == BadgeProtocol.BADGE_PARAGUACRAFT) {
                badge = BadgeProtocol.BADGE_PARAGUACRAFT;
            } else {
                badge = BadgeProtocol.BADGE_NONE;
            }
            if (badge == BadgeProtocol.BADGE_NONE) {
                return;
            }
            UUID id = player.getUniqueId();
            badges.setBadge(id, badge);
            badges.broadcastUpdate(id, badge);
        } catch (Exception ignored) {
        }
    }
}
