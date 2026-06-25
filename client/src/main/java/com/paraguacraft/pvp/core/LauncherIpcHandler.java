package com.paraguacraft.pvp.core;

import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import net.minecraftforge.fml.common.gameevent.TickEvent;

public class LauncherIpcHandler {

    private int tick;

    @SubscribeEvent
    public void onClientTick(TickEvent.ClientTickEvent event) {
        if (event.phase != TickEvent.Phase.END) {
            return;
        }
        tick++;
        if (tick >= 10) {
            tick = 0;
            LauncherIpc.poll();
        }
    }
}
