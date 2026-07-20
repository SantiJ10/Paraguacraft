package com.paraguacraft.pvp.modern.config;

import com.paraguacraft.pvp.modern.core.PerformanceConfig;
import com.paraguacraft.pvp.modern.gui.theme.MenuTheme;
import net.fabricmc.loader.api.FabricLoader;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Properties;

/** Lee tier de hardware y preset de rendimiento del launcher (`paraguacraft_modern.properties`). */
public final class LauncherProfile {

    public static String hardwareTier = "media";

    private LauncherProfile() {}

    public static void apply() {
        ModernConfig.load();
        Path path = gameDir().resolve("paraguacraft_modern.properties");
        if (!Files.isRegularFile(path)) {
            MenuTheme.setCurrent(MenuTheme.fromName(ModernConfig.menuTheme));
            return;
        }
        Properties props = new Properties();
        try (var in = Files.newInputStream(path)) {
            props.load(in);
            hardwareTier = props.getProperty("hardwareTier", hardwareTier);
            PerformanceConfig.loadTierFromProperties(props);
            MenuTheme.setCurrent(MenuTheme.fromName(ModernConfig.menuTheme));
        } catch (IOException ignored) {
        }
    }

    private static Path gameDir() {
        return FabricLoader.getInstance().getGameDir();
    }
}
