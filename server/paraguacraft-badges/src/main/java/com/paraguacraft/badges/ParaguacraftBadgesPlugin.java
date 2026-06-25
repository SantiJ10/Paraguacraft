package com.paraguacraft.badges;

import org.bukkit.plugin.java.JavaPlugin;

public final class ParaguacraftBadgesPlugin extends JavaPlugin {

    private static ParaguacraftBadgesPlugin instance;
    private BadgeService badgeService;

    @Override
    public void onEnable() {
        instance = this;
        badgeService = new BadgeService();
        BadgeListener listener = new BadgeListener(badgeService);
        getServer().getPluginManager().registerEvents(listener, this);
        getServer().getMessenger().registerOutgoingPluginChannel(this, BadgeProtocol.CHANNEL);
        getServer().getMessenger().registerIncomingPluginChannel(this, BadgeProtocol.CHANNEL, listener);
        getLogger().info("ParaguacraftBadges activo — canal " + BadgeProtocol.CHANNEL);
    }

    @Override
    public void onDisable() {
        getServer().getMessenger().unregisterOutgoingPluginChannel(this);
        getServer().getMessenger().unregisterIncomingPluginChannel(this);
        instance = null;
    }

    public static ParaguacraftBadgesPlugin getInstance() {
        return instance;
    }

    public BadgeService getBadgeService() {
        return badgeService;
    }
}
