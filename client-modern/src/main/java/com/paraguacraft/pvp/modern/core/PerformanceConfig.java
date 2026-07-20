package com.paraguacraft.pvp.modern.core;

import java.util.Properties;
import java.nio.file.Files;
import java.nio.file.Path;

/** Preset de rendimiento PvP modern (fase 4). El launcher escribe flags en `paraguacraft_modern.properties`. */
public final class PerformanceConfig {

    public enum ParticleMode {
        MINIMAL,
        REDUCED,
        ALL;

        public static ParticleMode fromName(String name) {
            if (name == null) {
                return MINIMAL;
            }
            for (ParticleMode mode : values()) {
                if (mode.name().equalsIgnoreCase(name.trim())) {
                    return mode;
                }
            }
            return MINIMAL;
        }
    }

    public static boolean boostFps = true;
    public static boolean applyVanillaPreset = true;
    public static boolean memoryCleanupOnWorldChange = true;
    public static boolean skipCombatFx = true;
    public static boolean reduceFpsWhenMinimized = true;
    public static int minimizedFps = 5;

    public static ParticleMode particleMode = ParticleMode.MINIMAL;
    public static int renderDistance = 10;
    public static int simulationDistance = 8;
    public static double entityDistanceScaling = 0.65;
    public static boolean oldAnimations = false;

    private PerformanceConfig() {}

    public static void loadFromProperties(Properties props) {
        loadTierFromProperties(props);
    }

    /** Solo distancias/partículas del launcher; boostFps vive en paraguacraftpvp-modern.properties. */
    public static void loadTierFromProperties(Properties props) {
        if (props.containsKey("boostFps") && !userConfigHasBoostFps()) {
            boostFps = Boolean.parseBoolean(props.getProperty("boostFps", "true"));
        }
        applyVanillaPreset = Boolean.parseBoolean(props.getProperty("applyVanillaPreset", "true"));
        memoryCleanupOnWorldChange = Boolean.parseBoolean(props.getProperty("memoryCleanup", "true"));
        skipCombatFx = Boolean.parseBoolean(props.getProperty("skipCombatFx", "true"));
        reduceFpsWhenMinimized = Boolean.parseBoolean(
            props.getProperty("reduceFpsWhenMinimized", "true")
        );
        minimizedFps = parseInt(props.getProperty("minimizedFps"), 5);
        particleMode = ParticleMode.fromName(props.getProperty("particleMode"));
        renderDistance = parseInt(props.getProperty("renderDistance"), renderDistance);
        simulationDistance = parseInt(props.getProperty("simulationDistance"), simulationDistance);
        entityDistanceScaling = parseDouble(
            props.getProperty("entityDistanceScaling"),
            entityDistanceScaling
        );
        oldAnimations = Boolean.parseBoolean(props.getProperty("oldAnimations", "false"));
    }

    private static boolean userConfigHasBoostFps() {
        Path path = net.fabricmc.loader.api.FabricLoader.getInstance()
            .getGameDir()
            .resolve("config")
            .resolve("paraguacraftpvp-modern.properties");
        if (!Files.isRegularFile(path)) {
            return false;
        }
        try {
            Properties p = new Properties();
            try (var in = Files.newInputStream(path)) {
                p.load(in);
            }
            return p.containsKey("boostFps");
        } catch (Exception e) {
            return false;
        }
    }

    private static int parseInt(String value, int fallback) {
        if (value == null) {
            return fallback;
        }
        try {
            return Integer.parseInt(value.trim());
        } catch (NumberFormatException e) {
            return fallback;
        }
    }

    private static double parseDouble(String value, double fallback) {
        if (value == null) {
            return fallback;
        }
        try {
            return Double.parseDouble(value.trim());
        } catch (NumberFormatException e) {
            return fallback;
        }
    }
}
