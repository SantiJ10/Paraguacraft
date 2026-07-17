package com.paraguacraft.pvp.modern.core;

import java.util.Properties;

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

    private PerformanceConfig() {}

    public static void loadFromProperties(Properties props) {
        boostFps = Boolean.parseBoolean(props.getProperty("boostFps", "true"));
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
