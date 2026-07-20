package com.paraguacraft.badges.paper;

import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerQuitEvent;
import org.bukkit.plugin.java.JavaPlugin;
import org.bukkit.plugin.messaging.PluginMessageListener;

import java.util.UUID;

/**
 * Puente servidor <-> cliente PvP Paraguacraft (1.8.9 y Modern) para insignias en el nametag.
 * Mismo protocolo que {@code BadgeProtocol}/{@code BadgeNetHandler} del lado cliente.
 */
public final class ParaguacraftBadgesPlugin extends JavaPlugin implements Listener, PluginMessageListener {

    private final BadgeRegistry registry = new BadgeRegistry();

    @Override
    public void onEnable() {
        getServer().getMessenger().registerOutgoingPluginChannel(this, BadgeProtocol.CHANNEL);
        getServer().getMessenger().registerIncomingPluginChannel(this, BadgeProtocol.CHANNEL, this);
        getServer().getPluginManager().registerEvents(this, this);
        getLogger().info("ParaguacraftBadges listo — canal " + BadgeProtocol.CHANNEL);
    }

    @Override
    public void onDisable() {
        getServer().getMessenger().unregisterOutgoingPluginChannel(this, BadgeProtocol.CHANNEL);
        getServer().getMessenger().unregisterIncomingPluginChannel(this, BadgeProtocol.CHANNEL, this);
        registry.clear();
    }

    @Override
    public void onPluginMessageReceived(String channel, Player player, byte[] message) {
        if (!BadgeProtocol.CHANNEL.equals(channel) || message.length < 1) {
            return;
        }
        byte type = message[0];
        if (type != BadgeProtocol.C2S_REGISTER) {
            return;
        }
        // El servidor decide la insignia final; nunca confía en el byte que pide el cliente
        // (salvo para saber que corre el mod). Esto evita que cualquiera se auto-asigne "staff".
        byte badge = player.hasPermission("paraguacraft.badge.staff")
            ? BadgeProtocol.BADGE_STAFF
            : BadgeProtocol.BADGE_PARAGUACRAFT;

        registry.set(player.getUniqueId(), badge);

        // El jugador que se registra recibe el estado completo actual.
        player.sendPluginMessage(this, BadgeProtocol.CHANNEL, BadgeMessages.sync(registry.snapshot()));

        // El resto de los jugadores (con el mod) recibe solo la actualización incremental.
        byte[] updatePayload = BadgeMessages.update(player.getUniqueId(), badge);
        for (Player online : getServer().getOnlinePlayers()) {
            if (!online.getUniqueId().equals(player.getUniqueId())) {
                online.sendPluginMessage(this, BadgeProtocol.CHANNEL, updatePayload);
            }
        }
    }

    @EventHandler
    public void onQuit(PlayerQuitEvent event) {
        UUID id = event.getPlayer().getUniqueId();
        if (!registry.has(id)) {
            return;
        }
        registry.remove(id);
        byte[] payload = BadgeMessages.update(id, BadgeProtocol.BADGE_NONE);
        for (Player online : getServer().getOnlinePlayers()) {
            online.sendPluginMessage(this, BadgeProtocol.CHANNEL, payload);
        }
    }
}
