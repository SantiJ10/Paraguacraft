package com.paraguacraft.pvp.modern.core;

import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;

/** Poll del archivo IPC del launcher (musica + hardware). */
public final class LauncherIpcHandler {

    private static int tick;

    private LauncherIpcHandler() {}

    public static void register() {
        ClientTickEvents.END_CLIENT_TICK.register(client -> {
            if (++tick % 10 == 0) {
                LauncherIpc.poll();
            }
        });
    }
}
