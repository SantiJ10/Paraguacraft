package com.paraguacraft.pvp.modern.core;

import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientLifecycleEvents;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.minecraft.client.MinecraftClient;

/** Actualiza Discord RPC cada ~2 s mientras el juego corre. */
public final class DiscordPresenceHandler {

    private static boolean registered;
    private static boolean connectStarted;
    private static int tickCounter;

    private DiscordPresenceHandler() {}

    public static void register() {
        if (registered || !DiscordPresence.isEnabled()) {
            return;
        }
        registered = true;
        ClientTickEvents.END_CLIENT_TICK.register(DiscordPresenceHandler::onEndTick);
        ClientLifecycleEvents.CLIENT_STOPPING.register(client -> DiscordPresence.disconnect());
    }

    private static void onEndTick(MinecraftClient client) {
        if (!DiscordPresence.isEnabled()) {
            return;
        }
        if (!connectStarted) {
            connectStarted = true;
            Thread t = new Thread(DiscordPresence::connect, "Paraguacraft-Discord");
            t.setDaemon(true);
            t.start();
        }
        tickCounter++;
        if (tickCounter >= 40) {
            tickCounter = 0;
            DiscordPresence.updateFromGame();
        }
    }
}
