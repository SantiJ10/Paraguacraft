package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.hud.MusicArtCache;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientLifecycleEvents;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.minecraft.client.MinecraftClient;

/** Hooks de sesion: perfiles por modo, shaders, quick play, deteccion de modo. */
public final class SessionBootstrap {

    private static boolean baselineCaptured;
    private static int detectCounter;

    private SessionBootstrap() {}

    public static void register() {
        ClientPlayConnectionEvents.JOIN.register((handler, sender, client) -> onJoin(client));
        ClientPlayConnectionEvents.DISCONNECT.register((handler, client) -> onDisconnect());
        ClientLifecycleEvents.CLIENT_STARTED.register(client -> {
            ChatTriggerConfig.ensureLoaded();
            ChatAlerts.ensureLoaded();
            MusicArtCache.preloadFromLauncherCache();
        });
        ClientTickEvents.END_CLIENT_TICK.register(SessionBootstrap::tick);
    }

    private static void onJoin(MinecraftClient client) {
        if (!baselineCaptured) {
            GameModeProfileManager.captureBaseline();
            baselineCaptured = true;
        }
        QuickPlayState.onJoin(client);
        ShaderAutoManager.onJoin(client);
        GameModeDetector.tick(client);
        GameModeProfileManager.onTick(client);
    }

    private static void onDisconnect() {
        ShaderAutoManager.onDisconnect();
        CombatStatsExporter.exportSession();
        GameModeProfileManager.reset();
        GameModeDetector.tick(null);
    }

    private static void tick(MinecraftClient client) {
        if (client.player == null) {
            return;
        }
        detectCounter++;
        if (detectCounter >= 20) {
            detectCounter = 0;
            GameModeDetector.tick(client);
            GameModeProfileManager.onTick(client);
            if (ModernConfig.shaderAutoOffInMatch && GameModeDetector.inMatch()) {
                ShaderAutoManager.onJoin(client);
            }
        }
        BridgeTimer.tick(client);
    }
}
