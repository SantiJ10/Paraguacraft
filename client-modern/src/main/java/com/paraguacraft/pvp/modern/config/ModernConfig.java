package com.paraguacraft.pvp.modern.config;

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
    public static boolean showPerfBadge = true;
    public static boolean showCoords = false;
    public static boolean showArmor = false;
    public static boolean showArmorPercentage = false;
    public static boolean showHeldItem = true;
    public static boolean showBedwarsResources = true;
    public static boolean showMusicHud = true;
    public static boolean showMusicAlbumArt = true;
    public static boolean showPotions = false;
    public static boolean showBlockOutline = true;
    public static boolean showCps = true;
    public static boolean toggleSprint = true;
    public static boolean toggleSprintLegacy = true;
    public static boolean pvpTrainingAutoWorld = false;
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
    public static int armorX = 5;
    public static int armorY = 200;
    public static int heldX = 5;
    public static int heldY = 140;
    public static int bwResX = 5;
    public static int bwResY = 320;
    public static int overlayHudX = 5;
    public static int overlayHudY = 120;
    public static int overlayHudW = 180;
    public static int musicHudAlpha = 180;
    public static String menuTheme = "CLASSIC";
    public static String customSkinUrl = "";

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
            showHeldItem = bool(props, "showHeldItem", showHeldItem);
            showBedwarsResources = bool(props, "showBedwarsResources", showBedwarsResources);
            showMusicHud = bool(props, "showMusicHud", showMusicHud);
            showMusicAlbumArt = bool(props, "showMusicAlbumArt", showMusicAlbumArt);
            showPotions = bool(props, "showPotions", showPotions);
            showBlockOutline = bool(props, "showBlockOutline", showBlockOutline);
            showCps = bool(props, "showCps", showCps);
            toggleSprint = bool(props, "toggleSprint", toggleSprint);
            toggleSprintLegacy = bool(props, "toggleSprintLegacy", toggleSprintLegacy);
            pvpTrainingAutoWorld = bool(props, "pvpTrainingAutoWorld", pvpTrainingAutoWorld);
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
            armorX = intProp(props, "armorX", armorX);
            armorY = intProp(props, "armorY", armorY);
            heldX = intProp(props, "heldX", heldX);
            heldY = intProp(props, "heldY", heldY);
            bwResX = intProp(props, "bwResX", bwResX);
            bwResY = intProp(props, "bwResY", bwResY);
            overlayHudX = intProp(props, "overlayHudX", overlayHudX);
            overlayHudY = intProp(props, "overlayHudY", overlayHudY);
            overlayHudW = intProp(props, "overlayHudW", overlayHudW);
            musicHudAlpha = intProp(props, "musicHudAlpha", musicHudAlpha);
            menuTheme = props.getProperty("menuTheme", menuTheme);
            customSkinUrl = props.getProperty("customSkinUrl", customSkinUrl);
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
        props.setProperty("showHeldItem", String.valueOf(showHeldItem));
        props.setProperty("showBedwarsResources", String.valueOf(showBedwarsResources));
        props.setProperty("showMusicHud", String.valueOf(showMusicHud));
        props.setProperty("showMusicAlbumArt", String.valueOf(showMusicAlbumArt));
        props.setProperty("showPotions", String.valueOf(showPotions));
        props.setProperty("showBlockOutline", String.valueOf(showBlockOutline));
        props.setProperty("showCps", String.valueOf(showCps));
        props.setProperty("toggleSprint", String.valueOf(toggleSprint));
        props.setProperty("toggleSprintLegacy", String.valueOf(toggleSprintLegacy));
        props.setProperty("pvpTrainingAutoWorld", String.valueOf(pvpTrainingAutoWorld));
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
        props.setProperty("heldX", String.valueOf(heldX));
        props.setProperty("heldY", String.valueOf(heldY));
        props.setProperty("bwResX", String.valueOf(bwResX));
        props.setProperty("bwResY", String.valueOf(bwResY));
        props.setProperty("overlayHudX", String.valueOf(overlayHudX));
        props.setProperty("overlayHudY", String.valueOf(overlayHudY));
        props.setProperty("overlayHudW", String.valueOf(overlayHudW));
        props.setProperty("musicHudAlpha", String.valueOf(musicHudAlpha));
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
