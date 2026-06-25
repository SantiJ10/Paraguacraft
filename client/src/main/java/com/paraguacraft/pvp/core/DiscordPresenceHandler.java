package com.paraguacraft.pvp.core;

import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import net.minecraftforge.fml.common.gameevent.TickEvent;

public class DiscordPresenceHandler {

    private int tickCounter;
    private boolean triedConnect;

    @SubscribeEvent
    public void onClientTick(TickEvent.ClientTickEvent event) {
        if (event.phase != TickEvent.Phase.END || !DiscordPresence.isEnabled()) {
            return;
        }
        if (!triedConnect) {
            triedConnect = true;
            DiscordPresence.connect();
        }
        tickCounter++;
        if (tickCounter >= 40) {
            tickCounter = 0;
            DiscordPresence.updateFromGame();
        }
    }
}
