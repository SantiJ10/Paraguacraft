package com.paraguacraft.pvp.core;

import java.util.Properties;

/**
 * Flags de rendimiento (Performance v2). Separados de {@code ModConfig} para no mezclar HUD/UI.
 */
public final class PerformanceConfig {

    public enum ParticleMode {
        OFF("Off", 0, 0, 2),
        MINIMAL("Minimal", 120, 15, 2),
        REDUCED("Reduced", 400, 50, 1),
        ALL("All", 800, 100, 0);

        private final String label;
        private final int maxActive;
        private final int maxPerTick;
        /** 0=All, 1=Decreased, 2=Minimal en {@code GameSettings.particleSetting}. */
        private final int vanillaParticles;

        ParticleMode(String label, int maxActive, int maxPerTick, int vanillaParticles) {
            this.label = label;
            this.maxActive = maxActive;
            this.maxPerTick = maxPerTick;
            this.vanillaParticles = vanillaParticles;
        }

        public String getLabel() {
            return label;
        }

        public int getVanillaParticles() {
            return vanillaParticles;
        }

        public ParticleMode next() {
            ParticleMode[] values = values();
            return values[(ordinal() + 1) % values.length];
        }

        public static ParticleMode fromName(String name) {
            if (name == null) {
                return REDUCED;
            }
            for (ParticleMode mode : values()) {
                if (mode.name().equalsIgnoreCase(name)) {
                    return mode;
                }
            }
            return REDUCED;
        }
    }

    /** Master toggle — aplica preset completo al activarse. */
    public static boolean boostFps = true;

    public static boolean entityCull = true;
    public static int entityCullDistanceSq = 48 * 48;

    public static boolean nametagCull = true;
    public static int nametagCullDistanceSq = 32 * 32;

    public static boolean entityAnimCull = true;
    public static int entityAnimCullDistanceSq = 28 * 28;

    public static boolean blockEntityCull = true;
    public static int blockEntityCullDistanceSq = 32 * 32;

    public static boolean particleLimit = true;
    public static ParticleMode particleMode = ParticleMode.REDUCED;
    public static int maxParticles = 400;
    public static int maxParticlesPerTick = 50;

    public static boolean memoryCleanupOnWorldChange = true;

    /** Aplica particulas/sombras/OptiFine al activar Boost FPS. */
    public static boolean applyVanillaPreset = true;

    /** Animaciones estilo 1.7 (blockhit, comer, swing). */
    public static boolean oldAnimations = true;

    public static boolean armorStandCull = true;
    public static int armorStandCullDistanceSq = 40 * 40;

    public static boolean itemFrameCull = true;
    public static int itemFrameCullDistanceSq = 32 * 32;

    /** Nametags lejanos solo si mirás al jugador (estilo Lunar LOD). */
    public static boolean nametagLod = true;
    public static int nametagLodDistanceSq = 18 * 18;

    /** Menos partículas de daño / muerte en combate. */
    public static boolean skipCombatFx = true;

    /** Aplica preset según RAM/CPU al primer arranque. */
    public static boolean hardwareAutoPreset = true;

    /**
     * Limita los FPS cuando la ventana está MINIMIZADA (estilo Lunar/Badlion).
     * Baja consumo de CPU/GPU y, en laptops, evita el thermal throttling que tira
     * los FPS al volver al juego. No afecta cuando la ventana está visible (sirve
     * para el borderless: podés mirar otra ventana al lado sin perder FPS).
     */
    public static boolean reduceFpsWhenMinimized = true;
    public static int minimizedFps = 5;

    /** Limita los FPS cuando la ventana NO tiene foco pero sigue visible (default off). */
    public static boolean reduceFpsWhenUnfocused = false;
    public static int unfocusedFps = 30;

    private PerformanceConfig() {}

    public static void setBoostFps(boolean enabled) {
        boostFps = enabled;
        if (enabled) {
            applyBoostPreset();
        }
    }

    /** Preset estilo Lunar/Badlion al encender Boost FPS. */
    public static void applyBoostPreset() {
        entityCull = true;
        nametagCull = true;
        entityAnimCull = true;
        blockEntityCull = true;
        particleLimit = true;
        particleMode = ParticleMode.REDUCED;
        memoryCleanupOnWorldChange = true;
        applyVanillaPreset = true;
        applyParticleLimitsFromMode();
        OptifinePreset.applyIfEnabled();
    }

    public static void cycleParticleMode() {
        particleMode = particleMode.next();
        applyParticleLimitsFromMode();
        if (particleMode == ParticleMode.OFF) {
            particleLimit = false;
        } else {
            particleLimit = true;
        }
        OptifinePreset.applyParticlesOnly();
    }

    public static void applyParticleLimitsFromMode() {
        if (particleMode == ParticleMode.OFF) {
            particleLimit = false;
            return;
        }
        particleLimit = true;
        maxParticles = particleMode.maxActive;
        maxParticlesPerTick = particleMode.maxPerTick;
    }

    public static void saveToProperties(Properties props) {
        props.setProperty("boostFps", String.valueOf(boostFps));
        props.setProperty("oldAnimations", String.valueOf(oldAnimations));
        props.setProperty("entityCull", String.valueOf(entityCull));
        props.setProperty("nametagCull", String.valueOf(nametagCull));
        props.setProperty("entityAnimCull", String.valueOf(entityAnimCull));
        props.setProperty("blockEntityCull", String.valueOf(blockEntityCull));
        props.setProperty("particleLimit", String.valueOf(particleLimit));
        props.setProperty("particleMode", particleMode.name());
        props.setProperty("memoryCleanup", String.valueOf(memoryCleanupOnWorldChange));
        props.setProperty("applyVanillaPreset", String.valueOf(applyVanillaPreset));
        props.setProperty("armorStandCull", String.valueOf(armorStandCull));
        props.setProperty("itemFrameCull", String.valueOf(itemFrameCull));
        props.setProperty("nametagLod", String.valueOf(nametagLod));
        props.setProperty("skipCombatFx", String.valueOf(skipCombatFx));
        props.setProperty("hardwareAutoPreset", String.valueOf(hardwareAutoPreset));
        props.setProperty("reduceFpsWhenMinimized", String.valueOf(reduceFpsWhenMinimized));
        props.setProperty("minimizedFps", String.valueOf(minimizedFps));
        props.setProperty("reduceFpsWhenUnfocused", String.valueOf(reduceFpsWhenUnfocused));
        props.setProperty("unfocusedFps", String.valueOf(unfocusedFps));
    }

    public static void loadFromProperties(Properties props) {
        boostFps = Boolean.parseBoolean(props.getProperty("boostFps", String.valueOf(boostFps)));
        oldAnimations = Boolean.parseBoolean(props.getProperty("oldAnimations", String.valueOf(oldAnimations)));
        entityCull = Boolean.parseBoolean(props.getProperty("entityCull", String.valueOf(entityCull)));
        nametagCull = Boolean.parseBoolean(props.getProperty("nametagCull", String.valueOf(nametagCull)));
        entityAnimCull = Boolean.parseBoolean(props.getProperty("entityAnimCull", String.valueOf(entityAnimCull)));
        blockEntityCull = Boolean.parseBoolean(props.getProperty("blockEntityCull", String.valueOf(blockEntityCull)));
        particleLimit = Boolean.parseBoolean(props.getProperty("particleLimit", String.valueOf(particleLimit)));
        particleMode = ParticleMode.fromName(props.getProperty("particleMode"));
        memoryCleanupOnWorldChange = Boolean.parseBoolean(
            props.getProperty("memoryCleanup", String.valueOf(memoryCleanupOnWorldChange))
        );
        applyVanillaPreset = Boolean.parseBoolean(
            props.getProperty("applyVanillaPreset", String.valueOf(applyVanillaPreset))
        );
        armorStandCull = Boolean.parseBoolean(props.getProperty("armorStandCull", String.valueOf(armorStandCull)));
        itemFrameCull = Boolean.parseBoolean(props.getProperty("itemFrameCull", String.valueOf(itemFrameCull)));
        nametagLod = Boolean.parseBoolean(props.getProperty("nametagLod", String.valueOf(nametagLod)));
        skipCombatFx = Boolean.parseBoolean(props.getProperty("skipCombatFx", String.valueOf(skipCombatFx)));
        hardwareAutoPreset = Boolean.parseBoolean(
            props.getProperty("hardwareAutoPreset", String.valueOf(hardwareAutoPreset))
        );
        reduceFpsWhenMinimized = Boolean.parseBoolean(
            props.getProperty("reduceFpsWhenMinimized", String.valueOf(reduceFpsWhenMinimized))
        );
        minimizedFps = parseIntSafe(props.getProperty("minimizedFps"), minimizedFps);
        reduceFpsWhenUnfocused = Boolean.parseBoolean(
            props.getProperty("reduceFpsWhenUnfocused", String.valueOf(reduceFpsWhenUnfocused))
        );
        unfocusedFps = parseIntSafe(props.getProperty("unfocusedFps"), unfocusedFps);
        applyParticleLimitsFromMode();
    }

    private static int parseIntSafe(String value, int fallback) {
        if (value == null) {
            return fallback;
        }
        try {
            return Integer.parseInt(value.trim());
        } catch (NumberFormatException e) {
            return fallback;
        }
    }
}
