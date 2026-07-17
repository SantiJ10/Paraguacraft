package com.paraguacraft.pvp.modern.config;

import net.fabricmc.loader.api.FabricLoader;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Properties;

/** Opciones persistentes del mod modern. */
public final class ModernConfig {

    public static boolean showFps = true;
    public static boolean showPing = true;
    public static boolean showKeystrokes = true;
    public static boolean showPerfBadge = true;
    public static boolean showCoords = false;
    public static boolean showArmor = false;
    public static boolean showCps = true;
    public static boolean toggleSprint = true;
    public static boolean pvpTrainingAutoWorld = false;
    public static int hudX = 4;
    public static int hudY = 4;
    public static String menuTheme = "CLASSIC";

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
            showFps = Boolean.parseBoolean(props.getProperty("showFps", "true"));
            showPing = Boolean.parseBoolean(props.getProperty("showPing", "true"));
            showKeystrokes = Boolean.parseBoolean(props.getProperty("showKeystrokes", "true"));
            showPerfBadge = Boolean.parseBoolean(props.getProperty("showPerfBadge", "true"));
            showCoords = Boolean.parseBoolean(props.getProperty("showCoords", "false"));
            showArmor = Boolean.parseBoolean(props.getProperty("showArmor", "false"));
            showCps = Boolean.parseBoolean(props.getProperty("showCps", "true"));
            toggleSprint = Boolean.parseBoolean(props.getProperty("toggleSprint", "true"));
            pvpTrainingAutoWorld = Boolean.parseBoolean(props.getProperty("pvpTrainingAutoWorld", "false"));
            hudX = Integer.parseInt(props.getProperty("hudX", "4"));
            hudY = Integer.parseInt(props.getProperty("hudY", "4"));
            menuTheme = props.getProperty("menuTheme", menuTheme);
        } catch (IOException | NumberFormatException ignored) {
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
        props.setProperty("showCps", String.valueOf(showCps));
        props.setProperty("toggleSprint", String.valueOf(toggleSprint));
        props.setProperty("pvpTrainingAutoWorld", String.valueOf(pvpTrainingAutoWorld));
        props.setProperty("hudX", String.valueOf(hudX));
        props.setProperty("hudY", String.valueOf(hudY));
        props.setProperty("menuTheme", menuTheme);
        Path path = configPath();
        try {
            Files.createDirectories(path.getParent());
            try (var out = Files.newOutputStream(path)) {
                props.store(out, "Paraguacraft PvP Modern");
            }
        } catch (IOException ignored) {
        }
    }
}
