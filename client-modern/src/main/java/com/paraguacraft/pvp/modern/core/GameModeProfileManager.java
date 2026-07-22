package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;

/** Aplica toggles HUD/QoL segun el modo detectado (BedWars, SkyWars, lobby...). */
public final class GameModeProfileManager {

    private static GameModeDetector.Mode lastApplied = GameModeDetector.Mode.LOBBY;
    private static boolean bedwarsResSaved;
    private static boolean blockCountSaved;
    private static boolean bridgeTimerSaved;

    private GameModeProfileManager() {}

    public static void onTick(MinecraftClient client) {
        if (!ModernConfig.autoGameModeProfiles || client == null) {
            return;
        }
        GameModeDetector.Mode mode = GameModeDetector.current();
        if (mode == lastApplied) {
            return;
        }
        apply(mode);
        lastApplied = mode;
    }

    private static void apply(GameModeDetector.Mode mode) {
        switch (mode) {
            case BEDWARS -> {
                ModernConfig.showBedwarsResources = true;
                ModernConfig.showBlockCount = true;
                ModernConfig.showBridgeTimer = true;
            }
            case SKYWARS, LUCKY_ISLANDS -> {
                ModernConfig.showBedwarsResources = false;
                ModernConfig.showBlockCount = true;
                ModernConfig.showBridgeTimer = true;
            }
            case DUELS, PVP -> {
                ModernConfig.showBedwarsResources = false;
                ModernConfig.showBlockCount = false;
                ModernConfig.showBridgeTimer = false;
            }
            case BUILD_BATTLE, TNT_RUN -> {
                ModernConfig.showBedwarsResources = false;
                ModernConfig.showBlockCount = true;
                ModernConfig.showBridgeTimer = false;
            }
            case LOBBY, OTHER -> restoreLobbyDefaults();
            default -> {}
        }
    }

    private static void restoreLobbyDefaults() {
        ModernConfig.showBedwarsResources = bedwarsResSaved;
        ModernConfig.showBlockCount = blockCountSaved;
        ModernConfig.showBridgeTimer = bridgeTimerSaved;
    }

    /** Guarda defaults del usuario al entrar al primer mundo. */
    public static void captureBaseline() {
        bedwarsResSaved = ModernConfig.showBedwarsResources;
        blockCountSaved = ModernConfig.showBlockCount;
        bridgeTimerSaved = ModernConfig.showBridgeTimer;
    }

    public static void reset() {
        lastApplied = GameModeDetector.Mode.LOBBY;
    }
}
