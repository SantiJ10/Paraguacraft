package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;

/**
 * Aplica solo HUD especifico del modo (BW recursos, altura, TNT, coords/brujula).
 * No pisa toggles del usuario (combo, armadura, reach, pociones, etc.).
 */
public final class GameModeProfileManager {

    private static GameModeDetector.Mode lastApplied = GameModeDetector.Mode.LOBBY;
    private static boolean bedwarsResSaved;
    private static boolean blockCountSaved;
    private static boolean bridgeTimerSaved;
    private static boolean coordsSaved;
    private static boolean compassSaved;
    private static boolean bwNamesSaved;
    private static boolean heightSaved;
    private static boolean tntSaved;

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
                ModernConfig.showItemNames = bwNamesSaved;
                ModernConfig.showBlockCount = true;
                ModernConfig.showBridgeTimer = true;
                ModernConfig.showCoords = false;
                ModernConfig.showCompass = true;
                ModernConfig.showHeightLimit = true;
                ModernConfig.showTntCountdown = true;
            }
            case SKYWARS, LUCKY_ISLANDS -> {
                ModernConfig.showBedwarsResources = false;
                ModernConfig.showBlockCount = true;
                ModernConfig.showBridgeTimer = true;
                ModernConfig.showCoords = false;
                ModernConfig.showCompass = true;
                ModernConfig.showHeightLimit = false;
            }
            case DUELS -> {
                ModernConfig.showBedwarsResources = false;
                ModernConfig.showBlockCount = false;
                ModernConfig.showBridgeTimer = false;
                ModernConfig.showCoords = false;
                ModernConfig.showCompass = false;
                ModernConfig.showHeightLimit = false;
            }
            case UHC -> {
                ModernConfig.showBedwarsResources = false;
                ModernConfig.showBlockCount = false;
                ModernConfig.showBridgeTimer = false;
                ModernConfig.showCoords = true;
                ModernConfig.showCompass = true;
                ModernConfig.showHeightLimit = false;
            }
            case PVP, HUNGER_GAMES -> {
                ModernConfig.showBedwarsResources = false;
                ModernConfig.showBlockCount = false;
                ModernConfig.showBridgeTimer = false;
                ModernConfig.showCoords = coordsSaved;
                ModernConfig.showCompass = compassSaved;
                ModernConfig.showHeightLimit = false;
            }
            case BUILD_BATTLE, TNT_RUN -> {
                ModernConfig.showBedwarsResources = false;
                ModernConfig.showBlockCount = true;
                ModernConfig.showBridgeTimer = false;
                ModernConfig.showCoords = true;
                ModernConfig.showCompass = true;
                ModernConfig.showHeightLimit = false;
            }
            case LOBBY, OTHER -> restoreLobbyDefaults();
            default -> {}
        }
    }

    private static void restoreLobbyDefaults() {
        ModernConfig.showBedwarsResources = bedwarsResSaved;
        ModernConfig.showBlockCount = blockCountSaved;
        ModernConfig.showBridgeTimer = bridgeTimerSaved;
        ModernConfig.showCoords = coordsSaved;
        ModernConfig.showCompass = compassSaved;
        ModernConfig.showItemNames = bwNamesSaved;
        ModernConfig.showHeightLimit = heightSaved;
        ModernConfig.showTntCountdown = tntSaved;
    }

    /** Guarda defaults del usuario al entrar al primer mundo. */
    public static void captureBaseline() {
        bedwarsResSaved = ModernConfig.showBedwarsResources;
        blockCountSaved = ModernConfig.showBlockCount;
        bridgeTimerSaved = ModernConfig.showBridgeTimer;
        coordsSaved = ModernConfig.showCoords;
        compassSaved = ModernConfig.showCompass;
        bwNamesSaved = ModernConfig.showItemNames;
        heightSaved = ModernConfig.showHeightLimit;
        tntSaved = ModernConfig.showTntCountdown;
    }

    public static void reset() {
        lastApplied = GameModeDetector.Mode.LOBBY;
    }
}
