package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;

/**
 * Aplica HUD/QoL segun el modo detectado (BedWars, SkyWars, Duels, HG…).
 * Solo toca toggles de juego; conserva baseline del usuario al volver al lobby.
 */
public final class GameModeProfileManager {

    private static GameModeDetector.Mode lastApplied = GameModeDetector.Mode.LOBBY;
    private static boolean bedwarsResSaved;
    private static boolean blockCountSaved;
    private static boolean bridgeTimerSaved;
    private static boolean armorSaved;
    private static boolean heldItemSaved;
    private static boolean potionsSaved;
    private static boolean coordsSaved;
    private static boolean comboSaved;
    private static boolean compassSaved;
    private static boolean bwNamesSaved;

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
                ModernConfig.showArmor = true;
                ModernConfig.showHeldItem = true;
                ModernConfig.showPotions = true;
                ModernConfig.showCoords = false;
                ModernConfig.comboCounter = true;
                ModernConfig.showCompass = true;
            }
            case SKYWARS, LUCKY_ISLANDS -> {
                ModernConfig.showBedwarsResources = false;
                ModernConfig.showBlockCount = true;
                ModernConfig.showBridgeTimer = true;
                ModernConfig.showArmor = true;
                ModernConfig.showHeldItem = heldItemSaved;
                ModernConfig.showPotions = true;
                ModernConfig.showCoords = false;
                ModernConfig.comboCounter = true;
                ModernConfig.showCompass = true;
            }
            case DUELS -> {
                // HUD mínimo, enfocado a pelea.
                ModernConfig.showBedwarsResources = false;
                ModernConfig.showBlockCount = false;
                ModernConfig.showBridgeTimer = false;
                ModernConfig.showArmor = true;
                ModernConfig.showHeldItem = true;
                ModernConfig.showPotions = true;
                ModernConfig.showCoords = false;
                ModernConfig.comboCounter = true;
                ModernConfig.showCompass = false;
            }
            case PVP, HUNGER_GAMES -> {
                ModernConfig.showBedwarsResources = false;
                ModernConfig.showBlockCount = false;
                ModernConfig.showBridgeTimer = false;
                ModernConfig.showArmor = true;
                ModernConfig.showHeldItem = true;
                ModernConfig.showPotions = true;
                ModernConfig.showCoords = coordsSaved;
                ModernConfig.comboCounter = true;
                ModernConfig.showCompass = compassSaved;
            }
            case BUILD_BATTLE, TNT_RUN -> {
                ModernConfig.showBedwarsResources = false;
                ModernConfig.showBlockCount = true;
                ModernConfig.showBridgeTimer = false;
                ModernConfig.showArmor = false;
                ModernConfig.showHeldItem = true;
                ModernConfig.showPotions = false;
                ModernConfig.showCoords = true;
                ModernConfig.comboCounter = false;
                ModernConfig.showCompass = true;
            }
            case LOBBY, OTHER -> restoreLobbyDefaults();
            default -> {}
        }
    }

    private static void restoreLobbyDefaults() {
        ModernConfig.showBedwarsResources = bedwarsResSaved;
        ModernConfig.showBlockCount = blockCountSaved;
        ModernConfig.showBridgeTimer = bridgeTimerSaved;
        ModernConfig.showArmor = armorSaved;
        ModernConfig.showHeldItem = heldItemSaved;
        ModernConfig.showPotions = potionsSaved;
        ModernConfig.showCoords = coordsSaved;
        ModernConfig.comboCounter = comboSaved;
        ModernConfig.showCompass = compassSaved;
        ModernConfig.showItemNames = bwNamesSaved;
    }

    /** Guarda defaults del usuario al entrar al primer mundo. */
    public static void captureBaseline() {
        bedwarsResSaved = ModernConfig.showBedwarsResources;
        blockCountSaved = ModernConfig.showBlockCount;
        bridgeTimerSaved = ModernConfig.showBridgeTimer;
        armorSaved = ModernConfig.showArmor;
        heldItemSaved = ModernConfig.showHeldItem;
        potionsSaved = ModernConfig.showPotions;
        coordsSaved = ModernConfig.showCoords;
        comboSaved = ModernConfig.comboCounter;
        compassSaved = ModernConfig.showCompass;
        bwNamesSaved = ModernConfig.showItemNames;
    }

    public static void reset() {
        lastApplied = GameModeDetector.Mode.LOBBY;
    }
}
