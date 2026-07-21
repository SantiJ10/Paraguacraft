package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.hud.MusicArtCache;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;

/** Poll del archivo IPC del launcher (musica + hardware). */
public final class LauncherIpcHandler {

    private LauncherIpcHandler() {}

    public static void register() {
        ClientTickEvents.END_CLIENT_TICK.register(client -> {
            LauncherIpc.poll();
            LauncherIpc.Snapshot snap = LauncherIpc.get();
            if (snap.valid && snap.musicPlaying && ModernConfig.showMusicAlbumArt
                && snap.musicImageUrl != null && !snap.musicImageUrl.isEmpty()) {
                MusicArtCache.get(snap.musicImageUrl);
            }
        });
    }
}
