package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;

/** Aplica toggles HUD/QoL segun el modo detectado (BedWars, SkyWars, lobby...). */
public final class GameModeProfileManager {

    private static GameModeDetector.Mode lastApplied = GameModeDetector.Mode.LOBBY;
    private static boolean bedwarsResSaved;
    private static boolean blockCountSaved;
    private static boolean bridgeTimerSaved;
    private static boolean armorSaved;
    private static boolean heldItemSaved;
    private static boolean potionsSaved;

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
                ModernConfig.showArmor = true;
                ModernConfig.showHeldItem = true;
                ModernConfig.showPotions = true;
            }
            case SKYWARS, LUCKY_ISLANDS -> {
                ModernConfig.showBedwarsResources = false;
                ModernConfig.showBlockCount = true;
                ModernConfig.showBridgeTimer = true;
                ModernConfig.showArmor = armorSaved;
                ModernConfig.showHeldItem = heldItemSaved;
                ModernConfig.showPotions = potionsSaved;
            }
            case DUELS, PVP -> {
                ModernConfig.showBedwarsResources = false;
                ModernConfig.showBlockCount = false;
                ModernConfig.showBridgeTimer = false;
                ModernConfig.showArmor = armorSaved;
                ModernConfig.showHeldItem = heldItemSaved;
                ModernConfig.showPotions = potionsSaved;
            }
            case BUILD_BATTLE, TNT_RUN -> {
                ModernConfig.showBedwarsResources = false;
                ModernConfig.showBlockCount = true;
                ModernConfig.showBridgeTimer = false;
                ModernConfig.showArmor = armorSaved;
                ModernConfig.showHeldItem = heldItemSaved;
                ModernConfig.showPotions = potionsSaved;
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
    }

    /** Guarda defaults del usuario al entrar al primer mundo. */
    public static void captureBaseline() {
        bedwarsResSaved = ModernConfig.showBedwarsResources;
        blockCountSaved = ModernConfig.showBlockCount;
        bridgeTimerSaved = ModernConfig.showBridgeTimer;
        armorSaved = ModernConfig.showArmor;
        heldItemSaved = ModernConfig.showHeldItem;
        potionsSaved = ModernConfig.showPotions;
    }

    public static void reset() {
        lastApplied = GameModeDetector.Mode.LOBBY;
    }
}
