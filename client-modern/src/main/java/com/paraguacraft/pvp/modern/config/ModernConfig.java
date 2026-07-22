package com.paraguacraft.pvp.modern.config;

import com.paraguacraft.pvp.modern.core.PerformanceConfig;
import net.fabricmc.loader.api.FabricLoader;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Properties;

/** Opciones persistentes del mod modern (HUD, QoL, posiciones). */
public final class ModernConfig {

    public static boolean showFps = true;
    public static boolean showPing = true;
    public static boolean showKeystrokes = true;
    public static boolean showPerfBadge = false;
    public static boolean showCoords = false;
    public static boolean showArmor = true;
    public static boolean showArmorPercentage = false;
    public static boolean showBlockCount = true;
    public static boolean showHeldItem = true;
    public static boolean showBedwarsResources = true;
    public static boolean bwResTransparentBg = true;
    public static boolean showMusicHud = true;
    public static boolean showMusicAlbumArt = true;
    public static boolean showPotions = true;
    public static boolean showBlockOutline = true;
    public static boolean showCps = true;
    public static boolean noHurtCam = true;
    public static boolean fullbright = false;
    public static boolean dynamicFov = true;
    public static boolean hideTitles = true;
    public static boolean scoreboardEnabled = true;
    public static boolean scoreboardTransparentBg = false;
    public static boolean scoreboardHideRedNumbers = false;
    public static boolean scoreboardHideStats = false;
    public static boolean lowFire = true;
    public static boolean itemPhysics = true;
    public static boolean oldAnimations = true;
    public static boolean comboCounter = true;
    public static boolean showTntCountdown = true;
    public static boolean chatTriggers = true;
    public static boolean chatAlertsEnabled = true;
    public static boolean freelookEnabled = true;
    /** Bloquea freelook en Hypixel/Cubecraft (reglas del servidor). */
    public static boolean freelookBlacklistServers = true;
    public static boolean shaderAutoOffInMatch = true;
    public static boolean reachDisplayPracticeOnly = false;
    public static boolean autoGameModeProfiles = true;
    public static boolean showGameModeHud = true;
    public static boolean showBridgeTimer = true;
    public static boolean showCompass = true;
    public static boolean showHardwareHud = true;
    public static boolean reachDisplay = true;
    /** 0=vanilla, 1=cruz, 2=gap, 3=punto, 4=icono */
    public static int crosshairMode = 0;
    public static int reachDisplayX = 5;
    public static int reachDisplayY = 58;
    public static String selectedResourcePack = "";
    /** Pack extra encima del oficial (no se pisa al reconectar). */
    public static String secondaryResourcePack = "";
    public static boolean nickFinderEnabled = true;
    public static String nickFinderQuery = "";
    public static boolean coloredBeds = true;
    public static boolean teamColors = true;
    public static boolean toggleSprint = true;
    public static boolean toggleSprintLegacy = false;
    /** Modo activado desde Mod Menu; el estado real vive en {@link #isSneakingToggled} (no persistido). */
    public static boolean toggleSneak = false;
    public static boolean isSneakingToggled = false;
    /** Culling (Fase 3 — paridad 1.8.9). Idle FPS vive en {@code PerformanceConfig.reduceFpsWhenMinimized}. */
    public static boolean entityCull = false;
    public static boolean nametagCull = false;
    /** Insignias + ping rival en nametags (Fase 3 — paridad social 1.8.9). */
    public static boolean showNametagLogo = true;
    public static boolean showNametagLogoOthers = true;
    public static boolean showOpponentPing = false;
    /** HUD con nombre/IP del servidor conectado (Fase 3 — paridad `drawServerHUD` 1.8.9). */
    public static boolean showServerHud = false;
    public static int serverHudX = 5;
    public static int serverHudY = 72;
    public static boolean windowedFullscreen = false;
    public static boolean pvpTrainingAutoWorld = false;
    /** Estadisticas de combate por sesion (Fase 5 — feature innovadora, ver CombatStats). */
    public static boolean showCombatStatsHud = false;
    public static int combatStatsX = 5;
    public static int combatStatsY = 185;
    public static int hudX = 4;
    public static int hudY = 4;
    public static int fpsX = 5;
    public static int fpsY = 5;
    public static int pingX = 5;
    public static int pingY = 18;
    public static int cpsX = 5;
    public static int cpsY = 31;
    public static int keysX = 5;
    public static int keysY = 55;
    /** Junto al hotbar, esquina inferior derecha (paridad visual con 1.8.9/vanilla). */
    public static int armorX = 300;
    public static int armorY = 222;
    public static int blocksX = 300;
    public static int blocksY = 200;
    /** Migracion one-shot del layout viejo (arriba a la izquierda) al nuevo. No se re-evalua despues. */
    public static boolean armorLayoutMigrated = false;
    private static final int OLD_ARMOR_X = 5;
    private static final int OLD_ARMOR_Y = 140;
    private static final int OLD_BLOCKS_X = 5;
    private static final int OLD_BLOCKS_Y = 260;
    public static int heldX = 5;
    public static int heldY = 140;
    public static int bwResX = 5;
    public static int bwResY = 320;
    public static int overlayHudX = 5;
    public static int overlayHudY = 260;
    public static int hardwareHudX = 5;
    public static int hardwareHudY = 260;
    public static int musicHudX = 5;
    public static int musicHudY = 308;
    public static int overlayHudW = 118;
    public static int musicHudAlpha = 255;
    /** Escala del panel de musica (100 = normal). */
    public static int musicHudScale = 100;
    public static int coordsX = 5;
    public static int coordsY = 44;
    private static final int[] MUSIC_ALPHA_PRESETS = {255, 64, 0};
    public static int comboX = 5;
    public static int comboY = 45;
    public static int gameModeHudX = 5;
    public static int gameModeHudY = 72;
    public static int bridgeTimerX = 5;
    public static int bridgeTimerY = 86;
    public static int potionX = 150;
    public static int potionY = 5;
    public static int compassY = 10;
    public static String menuTheme = "CLASSIC";
    public static String customSkinUrl = "";

    public static void cycleMusicHudAlpha() {
        int idx = 0;
        for (int i = 0; i < MUSIC_ALPHA_PRESETS.length; i++) {
            if (musicHudAlpha == MUSIC_ALPHA_PRESETS[i]) {
                idx = (i + 1) % MUSIC_ALPHA_PRESETS.length;
                break;
            }
        }
        musicHudAlpha = MUSIC_ALPHA_PRESETS[idx];
    }

    public static String musicHudAlphaLabel() {
        if (musicHudAlpha <= 0) {
            return "Transparente";
        }
        int pct = Math.round(musicHudAlpha / 255f * 100f);
        return pct + "%";
    }

    public static void cycleMusicHudScale() {
        int[] presets = {100, 125, 150, 175, 200};
        int idx = 0;
        for (int i = 0; i < presets.length; i++) {
            if (musicHudScale == presets[i]) {
                idx = (i + 1) % presets.length;
                break;
            }
        }
        musicHudScale = presets[idx];
    }

    public static String musicHudScaleLabel() {
        return musicHudScale + "%";
    }

    public static void cycleCrosshairMode() {
        crosshairMode = (crosshairMode + 1) % 5;
    }

    public static String crosshairModeLabel() {
        return switch (crosshairMode) {
            case 1 -> "Cruz";
            case 2 -> "Gap";
            case 3 -> "Punto";
            case 4 -> "Icono";
            default -> "Vanilla";
        };
    }

    private ModernConfig() {}

    private static Path configPath() {
        return FabricLoader.getInstance().getGameDir().resolve("config").resolve("paraguacraftpvp-modern.properties");
    }

    public static void load() {
        Path path = configPath();
        if (!Files.isRegularFile(path)) {
            return;
        }
        Properties props = new Properties();
        try (var in = Files.newInputStream(path)) {
            props.load(in);
            showFps = bool(props, "showFps", showFps);
            showPing = bool(props, "showPing", showPing);
            showKeystrokes = bool(props, "showKeystrokes", showKeystrokes);
            showPerfBadge = bool(props, "showPerfBadge", showPerfBadge);
            showCoords = bool(props, "showCoords", showCoords);
            showArmor = bool(props, "showArmor", showArmor);
            showArmorPercentage = bool(props, "showArmorPercentage", showArmorPercentage);
            showBlockCount = bool(props, "showBlockCount", showBlockCount);
            showHeldItem = bool(props, "showHeldItem", showHeldItem);
            showBedwarsResources = bool(props, "showBedwarsResources", showBedwarsResources);
            bwResTransparentBg = bool(props, "bwResTransparentBg", bwResTransparentBg);
            showMusicHud = bool(props, "showMusicHud", showMusicHud);
            showMusicAlbumArt = bool(props, "showMusicAlbumArt", showMusicAlbumArt);
            showPotions = bool(props, "showPotions", showPotions);
            showBlockOutline = bool(props, "showBlockOutline", showBlockOutline);
            showCps = bool(props, "showCps", showCps);
            noHurtCam = bool(props, "noHurtCam", noHurtCam);
            fullbright = bool(props, "fullbright", fullbright);
            dynamicFov = bool(props, "dynamicFov", dynamicFov);
            hideTitles = bool(props, "hideTitles", hideTitles);
            scoreboardEnabled = bool(props, "scoreboardEnabled", scoreboardEnabled);
            scoreboardTransparentBg = bool(props, "scoreboardTransparentBg", scoreboardTransparentBg);
            scoreboardHideRedNumbers = bool(props, "scoreboardHideRedNumbers", scoreboardHideRedNumbers);
            scoreboardHideStats = bool(props, "scoreboardHideStats", scoreboardHideStats);
            lowFire = bool(props, "lowFire", lowFire);
            itemPhysics = bool(props, "itemPhysics", itemPhysics);
            oldAnimations = bool(props, "oldAnimations", oldAnimations);
            comboCounter = bool(props, "comboCounter", comboCounter);
            showTntCountdown = bool(props, "showTntCountdown", showTntCountdown);
            chatTriggers = bool(props, "chatTriggers", chatTriggers);
            chatAlertsEnabled = bool(props, "chatAlertsEnabled", chatAlertsEnabled);
            freelookEnabled = bool(props, "freelookEnabled", freelookEnabled);
            freelookBlacklistServers = bool(props, "freelookBlacklistServers", freelookBlacklistServers);
            shaderAutoOffInMatch = bool(props, "shaderAutoOffInMatch", shaderAutoOffInMatch);
            reachDisplayPracticeOnly = bool(props, "reachDisplayPracticeOnly", reachDisplayPracticeOnly);
            autoGameModeProfiles = bool(props, "autoGameModeProfiles", autoGameModeProfiles);
            showGameModeHud = bool(props, "showGameModeHud", showGameModeHud);
            showBridgeTimer = bool(props, "showBridgeTimer", showBridgeTimer);
            showCompass = bool(props, "showCompass", showCompass);
            showHardwareHud = bool(props, "showHardwareHud", showHardwareHud);
            reachDisplay = bool(props, "reachDisplay", reachDisplay);
            crosshairMode = intProp(props, "crosshairMode", crosshairMode);
            reachDisplayX = intProp(props, "reachDisplayX", reachDisplayX);
            reachDisplayY = intProp(props, "reachDisplayY", reachDisplayY);
            selectedResourcePack = props.getProperty("selectedResourcePack", selectedResourcePack);
            if (selectedResourcePack == null) {
                selectedResourcePack = "";
            }
            secondaryResourcePack = props.getProperty("secondaryResourcePack", secondaryResourcePack);
            if (secondaryResourcePack == null) {
                secondaryResourcePack = "";
            }
            nickFinderEnabled = bool(props, "nickFinderEnabled", nickFinderEnabled);
            nickFinderQuery = props.getProperty("nickFinderQuery", nickFinderQuery);
            if (nickFinderQuery == null) {
                nickFinderQuery = "";
            }
            coloredBeds = bool(props, "coloredBeds", coloredBeds);
            teamColors = bool(props, "teamColors", teamColors);
            toggleSprint = bool(props, "toggleSprint", toggleSprint);
            toggleSprintLegacy = bool(props, "toggleSprintLegacy", toggleSprintLegacy);
            toggleSneak = bool(props, "toggleSneak", toggleSneak);
            entityCull = bool(props, "entityCull", entityCull);
            nametagCull = bool(props, "nametagCull", nametagCull);
            showNametagLogo = bool(props, "showNametagLogo", showNametagLogo);
            showNametagLogoOthers = bool(props, "showNametagLogoOthers", showNametagLogoOthers);
            showOpponentPing = bool(props, "showOpponentPing", showOpponentPing);
            showServerHud = bool(props, "showServerHud", showServerHud);
            serverHudX = intProp(props, "serverHudX", serverHudX);
            serverHudY = intProp(props, "serverHudY", serverHudY);
            windowedFullscreen = bool(props, "windowedFullscreen", windowedFullscreen);
            pvpTrainingAutoWorld = bool(props, "pvpTrainingAutoWorld", pvpTrainingAutoWorld);
            showCombatStatsHud = bool(props, "showCombatStatsHud", showCombatStatsHud);
            combatStatsX = intProp(props, "combatStatsX", combatStatsX);
            combatStatsY = intProp(props, "combatStatsY", combatStatsY);
            hudX = intProp(props, "hudX", hudX);
            hudY = intProp(props, "hudY", hudY);
            fpsX = intProp(props, "fpsX", fpsX);
            fpsY = intProp(props, "fpsY", fpsY);
            pingX = intProp(props, "pingX", pingX);
            pingY = intProp(props, "pingY", pingY);
            cpsX = intProp(props, "cpsX", cpsX);
            cpsY = intProp(props, "cpsY", cpsY);
            keysX = intProp(props, "keysX", keysX);
            keysY = intProp(props, "keysY", keysY);
            armorLayoutMigrated = bool(props, "armorLayoutMigrated", armorLayoutMigrated);
            armorX = intProp(props, "armorX", armorX);
            armorY = intProp(props, "armorY", armorY);
            blocksX = intProp(props, "blocksX", blocksX);
            blocksY = intProp(props, "blocksY", blocksY);
            if (!armorLayoutMigrated) {
                // Solo migra si nunca se tocaron a mano (siguen en el default viejo).
                // Si el jugador ya los arrastro a otro lado, se respeta tal cual.
                if (armorX == OLD_ARMOR_X && armorY == OLD_ARMOR_Y) {
                    armorX = 300;
                    armorY = 222;
                }
                if (blocksX == OLD_BLOCKS_X && blocksY == OLD_BLOCKS_Y) {
                    blocksX = 300;
                    blocksY = 200;
                }
                armorLayoutMigrated = true;
            }
            heldX = intProp(props, "heldX", heldX);
            heldY = intProp(props, "heldY", heldY);
            bwResX = intProp(props, "bwResX", bwResX);
            bwResY = intProp(props, "bwResY", bwResY);
            overlayHudX = intProp(props, "overlayHudX", overlayHudX);
            overlayHudY = intProp(props, "overlayHudY", overlayHudY);
            hardwareHudX = intProp(props, "hardwareHudX", overlayHudX);
            hardwareHudY = intProp(props, "hardwareHudY", overlayHudY);
            musicHudX = intProp(props, "musicHudX", overlayHudX);
            musicHudY = intProp(props, "musicHudY", overlayHudY + 48);
            overlayHudW = intProp(props, "overlayHudW", overlayHudW);
            if (overlayHudW > 140) {
                overlayHudW = 118;
            }
            musicHudAlpha = intProp(props, "musicHudAlpha", musicHudAlpha);
            musicHudScale = intProp(props, "musicHudScale", musicHudScale);
            coordsX = intProp(props, "coordsX", coordsX);
            coordsY = intProp(props, "coordsY", coordsY);
            comboX = intProp(props, "comboX", comboX);
            comboY = intProp(props, "comboY", comboY);
            gameModeHudX = intProp(props, "gameModeHudX", gameModeHudX);
            gameModeHudY = intProp(props, "gameModeHudY", gameModeHudY);
            bridgeTimerX = intProp(props, "bridgeTimerX", bridgeTimerX);
            bridgeTimerY = intProp(props, "bridgeTimerY", bridgeTimerY);
            potionX = intProp(props, "potionX", potionX);
            potionY = intProp(props, "potionY", potionY);
            compassY = intProp(props, "compassY", compassY);
            menuTheme = props.getProperty("menuTheme", menuTheme);
            customSkinUrl = props.getProperty("customSkinUrl", customSkinUrl);
            if (props.containsKey("boostFps")) {
                PerformanceConfig.boostFps = bool(props, "boostFps", PerformanceConfig.boostFps);
            }
            if (props.containsKey("reduceFpsWhenMinimized")) {
                PerformanceConfig.reduceFpsWhenMinimized =
                    bool(props, "reduceFpsWhenMinimized", PerformanceConfig.reduceFpsWhenMinimized);
            }
            if (props.containsKey("particleMode")) {
                PerformanceConfig.particleMode =
                    PerformanceConfig.ParticleMode.fromName(props.getProperty("particleMode"));
            }
        } catch (IOException ignored) {
        }
    }

    public static void save() {
        Properties props = new Properties();
        props.setProperty("showFps", String.valueOf(showFps));
        props.setProperty("showPing", String.valueOf(showPing));
        props.setProperty("showKeystrokes", String.valueOf(showKeystrokes));
        props.setProperty("showPerfBadge", String.valueOf(showPerfBadge));
        props.setProperty("showCoords", String.valueOf(showCoords));
        props.setProperty("showArmor", String.valueOf(showArmor));
        props.setProperty("showArmorPercentage", String.valueOf(showArmorPercentage));
        props.setProperty("showBlockCount", String.valueOf(showBlockCount));
        props.setProperty("showHeldItem", String.valueOf(showHeldItem));
        props.setProperty("showBedwarsResources", String.valueOf(showBedwarsResources));
        props.setProperty("bwResTransparentBg", String.valueOf(bwResTransparentBg));
        props.setProperty("showMusicHud", String.valueOf(showMusicHud));
        props.setProperty("showMusicAlbumArt", String.valueOf(showMusicAlbumArt));
        props.setProperty("showPotions", String.valueOf(showPotions));
        props.setProperty("showBlockOutline", String.valueOf(showBlockOutline));
        props.setProperty("showCps", String.valueOf(showCps));
        props.setProperty("noHurtCam", String.valueOf(noHurtCam));
        props.setProperty("fullbright", String.valueOf(fullbright));
        props.setProperty("dynamicFov", String.valueOf(dynamicFov));
        props.setProperty("hideTitles", String.valueOf(hideTitles));
        props.setProperty("scoreboardEnabled", String.valueOf(scoreboardEnabled));
        props.setProperty("scoreboardTransparentBg", String.valueOf(scoreboardTransparentBg));
        props.setProperty("scoreboardHideRedNumbers", String.valueOf(scoreboardHideRedNumbers));
        props.setProperty("scoreboardHideStats", String.valueOf(scoreboardHideStats));
        props.setProperty("lowFire", String.valueOf(lowFire));
        props.setProperty("itemPhysics", String.valueOf(itemPhysics));
        props.setProperty("oldAnimations", String.valueOf(oldAnimations));
        props.setProperty("comboCounter", String.valueOf(comboCounter));
        props.setProperty("showTntCountdown", String.valueOf(showTntCountdown));
        props.setProperty("chatTriggers", String.valueOf(chatTriggers));
        props.setProperty("chatAlertsEnabled", String.valueOf(chatAlertsEnabled));
        props.setProperty("freelookEnabled", String.valueOf(freelookEnabled));
        props.setProperty("freelookBlacklistServers", String.valueOf(freelookBlacklistServers));
        props.setProperty("shaderAutoOffInMatch", String.valueOf(shaderAutoOffInMatch));
        props.setProperty("reachDisplayPracticeOnly", String.valueOf(reachDisplayPracticeOnly));
        props.setProperty("autoGameModeProfiles", String.valueOf(autoGameModeProfiles));
        props.setProperty("showGameModeHud", String.valueOf(showGameModeHud));
        props.setProperty("showBridgeTimer", String.valueOf(showBridgeTimer));
        props.setProperty("showCompass", String.valueOf(showCompass));
        props.setProperty("showHardwareHud", String.valueOf(showHardwareHud));
        props.setProperty("reachDisplay", String.valueOf(reachDisplay));
        props.setProperty("crosshairMode", String.valueOf(crosshairMode));
        props.setProperty("reachDisplayX", String.valueOf(reachDisplayX));
        props.setProperty("reachDisplayY", String.valueOf(reachDisplayY));
        props.setProperty("selectedResourcePack", selectedResourcePack == null ? "" : selectedResourcePack);
        props.setProperty("secondaryResourcePack", secondaryResourcePack == null ? "" : secondaryResourcePack);
        props.setProperty("nickFinderEnabled", String.valueOf(nickFinderEnabled));
        props.setProperty("nickFinderQuery", nickFinderQuery == null ? "" : nickFinderQuery);
        props.setProperty("coloredBeds", String.valueOf(coloredBeds));
        props.setProperty("teamColors", String.valueOf(teamColors));
        props.setProperty("boostFps", String.valueOf(PerformanceConfig.boostFps));
        props.setProperty("reduceFpsWhenMinimized", String.valueOf(PerformanceConfig.reduceFpsWhenMinimized));
        props.setProperty("particleMode", PerformanceConfig.particleMode.name());
        props.setProperty("toggleSprint", String.valueOf(toggleSprint));
        props.setProperty("toggleSprintLegacy", String.valueOf(toggleSprintLegacy));
        props.setProperty("toggleSneak", String.valueOf(toggleSneak));
        props.setProperty("entityCull", String.valueOf(entityCull));
        props.setProperty("nametagCull", String.valueOf(nametagCull));
        props.setProperty("showNametagLogo", String.valueOf(showNametagLogo));
        props.setProperty("showNametagLogoOthers", String.valueOf(showNametagLogoOthers));
        props.setProperty("showOpponentPing", String.valueOf(showOpponentPing));
        props.setProperty("showServerHud", String.valueOf(showServerHud));
        props.setProperty("serverHudX", String.valueOf(serverHudX));
        props.setProperty("serverHudY", String.valueOf(serverHudY));
        props.setProperty("windowedFullscreen", String.valueOf(windowedFullscreen));
        props.setProperty("pvpTrainingAutoWorld", String.valueOf(pvpTrainingAutoWorld));
        props.setProperty("showCombatStatsHud", String.valueOf(showCombatStatsHud));
        props.setProperty("combatStatsX", String.valueOf(combatStatsX));
        props.setProperty("combatStatsY", String.valueOf(combatStatsY));
        props.setProperty("hudX", String.valueOf(hudX));
        props.setProperty("hudY", String.valueOf(hudY));
        props.setProperty("fpsX", String.valueOf(fpsX));
        props.setProperty("fpsY", String.valueOf(fpsY));
        props.setProperty("pingX", String.valueOf(pingX));
        props.setProperty("pingY", String.valueOf(pingY));
        props.setProperty("cpsX", String.valueOf(cpsX));
        props.setProperty("cpsY", String.valueOf(cpsY));
        props.setProperty("keysX", String.valueOf(keysX));
        props.setProperty("keysY", String.valueOf(keysY));
        props.setProperty("armorX", String.valueOf(armorX));
        props.setProperty("armorY", String.valueOf(armorY));
        props.setProperty("blocksX", String.valueOf(blocksX));
        props.setProperty("blocksY", String.valueOf(blocksY));
        props.setProperty("armorLayoutMigrated", String.valueOf(armorLayoutMigrated));
        props.setProperty("heldX", String.valueOf(heldX));
        props.setProperty("heldY", String.valueOf(heldY));
        props.setProperty("bwResX", String.valueOf(bwResX));
        props.setProperty("bwResY", String.valueOf(bwResY));
        props.setProperty("overlayHudX", String.valueOf(overlayHudX));
        props.setProperty("overlayHudY", String.valueOf(overlayHudY));
        props.setProperty("hardwareHudX", String.valueOf(hardwareHudX));
        props.setProperty("hardwareHudY", String.valueOf(hardwareHudY));
        props.setProperty("musicHudX", String.valueOf(musicHudX));
        props.setProperty("musicHudY", String.valueOf(musicHudY));
        props.setProperty("overlayHudW", String.valueOf(overlayHudW));
        props.setProperty("musicHudAlpha", String.valueOf(musicHudAlpha));
        props.setProperty("musicHudScale", String.valueOf(musicHudScale));
        props.setProperty("coordsX", String.valueOf(coordsX));
        props.setProperty("coordsY", String.valueOf(coordsY));
        props.setProperty("comboX", String.valueOf(comboX));
        props.setProperty("comboY", String.valueOf(comboY));
        props.setProperty("gameModeHudX", String.valueOf(gameModeHudX));
        props.setProperty("gameModeHudY", String.valueOf(gameModeHudY));
        props.setProperty("bridgeTimerX", String.valueOf(bridgeTimerX));
        props.setProperty("bridgeTimerY", String.valueOf(bridgeTimerY));
        props.setProperty("potionX", String.valueOf(potionX));
        props.setProperty("potionY", String.valueOf(potionY));
        props.setProperty("compassY", String.valueOf(compassY));
        props.setProperty("menuTheme", menuTheme);
        props.setProperty("customSkinUrl", customSkinUrl == null ? "" : customSkinUrl);
        Path path = configPath();
        try {
            Files.createDirectories(path.getParent());
            try (var out = Files.newOutputStream(path)) {
                props.store(out, "Paraguacraft PvP Modern");
            }
        } catch (IOException ignored) {
        }
    }

    private static boolean bool(Properties props, String key, boolean fallback) {
        return Boolean.parseBoolean(props.getProperty(key, String.valueOf(fallback)));
    }

    private static int intProp(Properties props, String key, int fallback) {
        try {
            return Integer.parseInt(props.getProperty(key, String.valueOf(fallback)));
        } catch (NumberFormatException e) {
            return fallback;
        }
    }
}
