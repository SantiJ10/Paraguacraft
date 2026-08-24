package com.paraguacraft.pvp.core;

import com.paraguacraft.pvp.hud.HUDOverlay;
import com.paraguacraft.pvp.hud.MusicArtCache;

/**
 * Libera caches del cliente al descargar un mundo (lobbies / partidas).
 * No llama a {@code System.gc()} en juego: G1 del launcher cubre el heap
 * y un GC explícito provoca stuttering en el input.
 */
public final class MemoryCleanup {

    private MemoryCleanup() {}

    public static void onWorldUnload() {
        HUDOverlay.clearCaches();
        MusicArtCache.clear();
    }
}
