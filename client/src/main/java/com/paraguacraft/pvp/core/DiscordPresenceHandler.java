package com.paraguacraft.pvp.core;

import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import net.minecraftforge.fml.common.gameevent.TickEvent;

public class DiscordPresenceHandler {

    private static volatile boolean connectStarted;
    private int tickCounter;

    @SubscribeEvent
    public void onClientTick(TickEvent.ClientTickEvent event) {
        if (event.phase != TickEvent.Phase.END || !DiscordPresence.isEnabled()) {
            return;
        }
        if (!connectStarted) {
            connectStarted = true;
            Thread t = new Thread(new Runnable() {
                @Override
                public void run() {
                    DiscordPresence.connect();
                }
            }, "Paraguacraft-Discord");
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
