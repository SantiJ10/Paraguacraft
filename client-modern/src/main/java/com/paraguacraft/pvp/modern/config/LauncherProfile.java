package com.paraguacraft.pvp.modern.config;

import com.paraguacraft.pvp.modern.core.PerformanceConfig;
import com.paraguacraft.pvp.modern.gui.theme.MenuTheme;
import net.fabricmc.loader.api.FabricLoader;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Properties;

/** Lee `paraguacraft_modern.properties` que escribe el launcher al lanzar. */
public final class LauncherProfile {

    public static String hardwareTier = "media";

    private LauncherProfile() {}

    public static void apply() {
        ModernConfig.load();
        Path path = gameDir().resolve("paraguacraft_modern.properties");
        if (!Files.isRegularFile(path)) {
            return;
        }
        Properties props = new Properties();
        try (var in = Files.newInputStream(path)) {
            props.load(in);
            if (props.containsKey("showFps")) {
                ModernConfig.showFps = Boolean.parseBoolean(props.getProperty("showFps"));
            }
            if (props.containsKey("showPing")) {
                ModernConfig.showPing = Boolean.parseBoolean(props.getProperty("showPing"));
            }
            if (props.containsKey("showKeystrokes")) {
                ModernConfig.showKeystrokes = Boolean.parseBoolean(props.getProperty("showKeystrokes"));
            }
            if (props.containsKey("showPerfBadge")) {
                ModernConfig.showPerfBadge = Boolean.parseBoolean(props.getProperty("showPerfBadge"));
            }
            if (props.containsKey("showCoords")) {
                ModernConfig.showCoords = Boolean.parseBoolean(props.getProperty("showCoords"));
            }
            if (props.containsKey("showArmor")) {
                ModernConfig.showArmor = Boolean.parseBoolean(props.getProperty("showArmor"));
            }
            if (props.containsKey("showCps")) {
                ModernConfig.showCps = Boolean.parseBoolean(props.getProperty("showCps"));
            }
            if (props.containsKey("hudX")) {
                ModernConfig.hudX = Integer.parseInt(props.getProperty("hudX"));
            }
            if (props.containsKey("hudY")) {
                ModernConfig.hudY = Integer.parseInt(props.getProperty("hudY"));
            }
            if (props.containsKey("menuTheme")) {
                ModernConfig.menuTheme = props.getProperty("menuTheme");
            }
            if (props.containsKey("pvpTrainingAutoWorld")) {
                ModernConfig.pvpTrainingAutoWorld = Boolean.parseBoolean(props.getProperty("pvpTrainingAutoWorld"));
            }
            if (props.containsKey("toggleSprint")) {
                ModernConfig.toggleSprint = Boolean.parseBoolean(props.getProperty("toggleSprint"));
            }
            hardwareTier = props.getProperty("hardwareTier", hardwareTier);
            PerformanceConfig.loadFromProperties(props);
            MenuTheme.setCurrent(MenuTheme.fromName(ModernConfig.menuTheme));
        } catch (IOException | NumberFormatException ignored) {
        }
    }

    private static Path gameDir() {
        return FabricLoader.getInstance().getGameDir();
    }
}
