package com.paraguacraft.pvp.core;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import net.minecraftforge.fml.common.gameevent.TickEvent;

/**
 * Aplica HUD segun el modo detectado. Conserva baseline al volver al lobby.
 * Reemplaza el auto-HUD solo-BedWars de {@link BedwarsModeHelper}.
 */
public final class GameModeProfileManager {

    private static GameModeDetector.Mode lastApplied = GameModeDetector.Mode.LOBBY;
    private static boolean baselineCaptured;
    private static boolean bedwarsResSaved;
    private static boolean blockCountSaved;
    private static boolean armorSaved;
    private static boolean heldItemSaved;
    private static boolean potionsSaved;
    private static boolean coordsSaved;
    private static boolean comboSaved;
    private static boolean compassSaved;
    private static boolean heightSaved;
    private static boolean opponentPingSaved;
    private static int tickCounter;

    public GameModeProfileManager() {}

    @SubscribeEvent
    public void onClientTick(TickEvent.ClientTickEvent event) {
        if (event.phase != TickEvent.Phase.END) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc == null || mc.theWorld == null || mc.thePlayer == null) {
            if (lastApplied != GameModeDetector.Mode.LOBBY && ModConfig.autoGameModeProfiles) {
                restoreLobbyDefaults();
                lastApplied = GameModeDetector.Mode.LOBBY;
            }
            GameModeDetector.tick();
            return;
        }
        if (!baselineCaptured) {
            captureBaseline();
        }
        tickCounter++;
        if (tickCounter < 20) {
            return;
        }
        tickCounter = 0;
        GameModeDetector.tick();
        onTick();
    }

    public static void onTick() {
        if (!ModConfig.autoGameModeProfiles) {
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
            case BEDWARS:
                ModConfig.showBedwarsResources = true;
                ModConfig.showBlockCount = true;
                ModConfig.showArmor = true;
                ModConfig.showHeldItem = true;
                ModConfig.showPotions = true;
                ModConfig.showCoords = false;
                ModConfig.comboCounter = true;
                ModConfig.showCompass = true;
                ModConfig.showHeightLimit = true;
                ModConfig.showTntCountdown = true;
                break;
            case SKYWARS:
            case LUCKY_ISLANDS:
                ModConfig.showBedwarsResources = false;
                ModConfig.showBlockCount = true;
                ModConfig.showArmor = true;
                ModConfig.showHeldItem = heldItemSaved;
                ModConfig.showPotions = true;
                ModConfig.showCoords = false;
                ModConfig.comboCounter = true;
                ModConfig.showCompass = true;
                ModConfig.showHeightLimit = false;
                break;
            case DUELS:
                ModConfig.showBedwarsResources = false;
                ModConfig.showBlockCount = false;
                ModConfig.showArmor = true;
                ModConfig.showHeldItem = true;
                ModConfig.showPotions = true;
                ModConfig.showCoords = false;
                ModConfig.comboCounter = true;
                ModConfig.showCompass = false;
                ModConfig.showHeightLimit = false;
                ModConfig.showOpponentPing = true;
                ModConfig.reachDisplay = true;
                break;
            case UHC:
                ModConfig.showBedwarsResources = false;
                ModConfig.showBlockCount = false;
                ModConfig.showArmor = true;
                ModConfig.showHeldItem = true;
                ModConfig.showPotions = true;
                ModConfig.showCoords = true;
                ModConfig.comboCounter = true;
                ModConfig.showCompass = true;
                ModConfig.showHeightLimit = false;
                break;
            case PVP:
            case HUNGER_GAMES:
                ModConfig.showBedwarsResources = false;
                ModConfig.showBlockCount = false;
                ModConfig.showArmor = true;
                ModConfig.showHeldItem = true;
                ModConfig.showPotions = true;
                ModConfig.showCoords = coordsSaved;
                ModConfig.comboCounter = true;
                ModConfig.showCompass = compassSaved;
                ModConfig.showHeightLimit = false;
                break;
            case BUILD_BATTLE:
            case TNT_RUN:
                ModConfig.showBedwarsResources = false;
                ModConfig.showBlockCount = true;
                ModConfig.showArmor = false;
                ModConfig.showHeldItem = true;
                ModConfig.showPotions = false;
                ModConfig.showCoords = true;
                ModConfig.comboCounter = false;
                ModConfig.showCompass = true;
                ModConfig.showHeightLimit = false;
                break;
            case LOBBY:
            case OTHER:
            default:
                restoreLobbyDefaults();
                break;
        }
    }

    private static void restoreLobbyDefaults() {
        ModConfig.showBedwarsResources = bedwarsResSaved;
        ModConfig.showBlockCount = blockCountSaved;
        ModConfig.showArmor = armorSaved;
        ModConfig.showHeldItem = heldItemSaved;
        ModConfig.showPotions = potionsSaved;
        ModConfig.showCoords = coordsSaved;
        ModConfig.comboCounter = comboSaved;
        ModConfig.showCompass = compassSaved;
        ModConfig.showHeightLimit = heightSaved;
        ModConfig.showOpponentPing = opponentPingSaved;
    }

    public static void captureBaseline() {
        bedwarsResSaved = ModConfig.showBedwarsResources;
        blockCountSaved = ModConfig.showBlockCount;
        armorSaved = ModConfig.showArmor;
        heldItemSaved = ModConfig.showHeldItem;
        potionsSaved = ModConfig.showPotions;
        coordsSaved = ModConfig.showCoords;
        comboSaved = ModConfig.comboCounter;
        compassSaved = ModConfig.showCompass;
        heightSaved = ModConfig.showHeightLimit;
        opponentPingSaved = ModConfig.showOpponentPing;
        baselineCaptured = true;
    }

    public static void reset() {
        lastApplied = GameModeDetector.Mode.LOBBY;
        baselineCaptured = false;
    }
}
