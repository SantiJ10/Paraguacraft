package com.paraguacraft.pvp.core;

import com.sun.management.OperatingSystemMXBean;

import java.lang.management.ManagementFactory;

/** Detecta hardware y aplica preset de rendimiento (Fase C). */
public final class HardwarePreset {

    public enum Tier {
        LOW, MEDIUM, HIGH
    }

    private static Tier detectedTier = Tier.MEDIUM;
    private static boolean applied;

    private HardwarePreset() {}

    public static Tier detectTier() {
        long maxMb = Runtime.getRuntime().maxMemory() / (1024L * 1024L);
        int cores = Runtime.getRuntime().availableProcessors();
        long totalRamMb = maxMb;
        try {
            OperatingSystemMXBean os = (OperatingSystemMXBean) ManagementFactory.getOperatingSystemMXBean();
            totalRamMb = os.getTotalPhysicalMemorySize() / (1024L * 1024L);
        } catch (Exception ignored) {
        }
        if (totalRamMb <= 8192 || cores <= 4) {
            detectedTier = Tier.LOW;
        } else if (totalRamMb <= 16384 || cores <= 8) {
            detectedTier = Tier.MEDIUM;
        } else {
            detectedTier = Tier.HIGH;
        }
        return detectedTier;
    }

    public static void applyIfEnabled() {
        if (!PerformanceConfig.hardwareAutoPreset || applied) {
            return;
        }
        applied = true;
        Tier tier = detectTier();
        // Estilo competitive (launcher / property): prioridad full FPS even on HIGH.
        if (isCompetitiveStyle()) {
            applyCompetitive(tier);
        } else {
            switch (tier) {
                case LOW:
                    applyLow();
                    break;
                case HIGH:
                    applyHigh();
                    break;
                default:
                    applyMedium();
                    break;
            }
        }
        System.out.println("[Paraguacraft V2] Hardware preset: " + tier.name()
            + " style=" + (isCompetitiveStyle() ? "competitive" : "casual"));
    }

    private static boolean isCompetitiveStyle() {
        try {
            java.io.File f = new java.io.File(
                net.minecraft.client.Minecraft.getMinecraft().mcDataDir,
                "paraguacraft_v2.properties"
            );
            if (!f.isFile()) {
                return true; // default full rendimiento
            }
            java.util.Properties p = new java.util.Properties();
            java.io.FileInputStream in = new java.io.FileInputStream(f);
            p.load(in);
            in.close();
            String s = p.getProperty("playStyle", "competitive");
            return !"casual".equalsIgnoreCase(s) && !"relaxed".equalsIgnoreCase(s);
        } catch (Exception e) {
            return true;
        }
    }

    private static void applyCompetitive(Tier hardware) {
        PerformanceConfig.boostFps = true;
        PerformanceConfig.applyBoostPreset();
        PerformanceConfig.particleMode = PerformanceConfig.ParticleMode.MINIMAL;
        PerformanceConfig.oldAnimations = true;
        PerformanceConfig.skipCombatFx = true;
        if (hardware == Tier.LOW) {
            PerformanceConfig.entityCullDistanceSq = 40 * 40;
            PerformanceConfig.nametagCullDistanceSq = 24 * 24;
            PerformanceConfig.armorStandCullDistanceSq = 32 * 32;
            PerformanceConfig.itemFrameCullDistanceSq = 24 * 24;
        } else {
            PerformanceConfig.entityCullDistanceSq = 48 * 48;
            PerformanceConfig.nametagCullDistanceSq = 32 * 32;
        }
        PerformanceConfig.applyParticleLimitsFromMode();
        OptifinePreset.applyIfEnabled();
    }

    private static void applyLow() {
        PerformanceConfig.boostFps = true;
        PerformanceConfig.applyBoostPreset();
        PerformanceConfig.particleMode = PerformanceConfig.ParticleMode.MINIMAL;
        PerformanceConfig.entityCullDistanceSq = 40 * 40;
        PerformanceConfig.nametagCullDistanceSq = 24 * 24;
        PerformanceConfig.armorStandCullDistanceSq = 32 * 32;
        PerformanceConfig.itemFrameCullDistanceSq = 24 * 24;
        PerformanceConfig.applyParticleLimitsFromMode();
    }

    private static void applyMedium() {
        PerformanceConfig.boostFps = true;
        PerformanceConfig.applyBoostPreset();
        PerformanceConfig.particleMode = PerformanceConfig.ParticleMode.REDUCED;
        PerformanceConfig.applyParticleLimitsFromMode();
    }

    private static void applyHigh() {
        PerformanceConfig.boostFps = true;
        PerformanceConfig.entityCull = true;
        PerformanceConfig.nametagCull = true;
        PerformanceConfig.particleMode = PerformanceConfig.ParticleMode.REDUCED;
        PerformanceConfig.applyParticleLimitsFromMode();
    }

    public static Tier getDetectedTier() {
        return detectedTier;
    }
}
