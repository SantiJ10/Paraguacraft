package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.LauncherProfile;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.AttackIndicator;
import net.minecraft.client.option.CloudRenderMode;
import net.minecraft.client.option.GameOptions;
import net.minecraft.client.option.GraphicsMode;
import net.minecraft.particle.ParticlesMode;

/**
 * Estilo de juego PvP: Casual vs Full rendimiento (competitive).
 * Ajuste fino para sentir más “1.8.9” en 1.21 (misma FPS, menos “pesadez”).
 */
public final class PlayStyle {

    public enum Kind {
        CASUAL,
        COMPETITIVE
    }

    private PlayStyle() {}

    public static Kind current() {
        String s = LauncherProfile.playStyle;
        if (s == null) {
            return Kind.COMPETITIVE;
        }
        return switch (s.trim().toLowerCase()) {
            case "casual", "relaxed", "chill" -> Kind.CASUAL;
            default -> Kind.COMPETITIVE;
        };
    }

    public static boolean isCompetitive() {
        return current() == Kind.COMPETITIVE;
    }

    /** Aplica flags del mod + options de feel (cada arranque). */
    public static void apply(MinecraftClient client) {
        if (client == null || client.options == null) {
            return;
        }
        Kind kind = current();
        if (kind == Kind.COMPETITIVE) {
            applyCompetitiveModFlags();
            applyCompetitiveOptions(client.options);
        } else {
            applyCasualModFlags();
            applyCasualOptions(client.options);
        }
        try {
            client.options.write();
        } catch (Exception ignored) {
        }
    }

    private static void applyCompetitiveModFlags() {
        PerformanceConfig.boostFps = true;
        PerformanceConfig.applyVanillaPreset = true;
        PerformanceConfig.skipCombatFx = true;
        PerformanceConfig.oldAnimations = true;
        PerformanceConfig.particleMode = PerformanceConfig.ParticleMode.MINIMAL;
        // Distancias “tipo 1.8”: menos world tick/render carga → menos “lag floaty”.
        if (PerformanceConfig.renderDistance > 10) {
            PerformanceConfig.renderDistance = 10;
        }
        if (PerformanceConfig.simulationDistance > 8) {
            PerformanceConfig.simulationDistance = 8;
        }
        if (PerformanceConfig.entityDistanceScaling > 0.65) {
            PerformanceConfig.entityDistanceScaling = 0.65;
        }

        ModernConfig.oldAnimations = true;
        ModernConfig.dynamicFov = false;
        ModernConfig.noHurtCam = true;
        ModernConfig.lowFire = true;
        ModernConfig.shaderAutoOffInMatch = true;
        ModernConfig.itemPhysics = false;
        // Cull on — menos entidades por frame
        ModernConfig.entityCull = true;
        ModernConfig.nametagCull = true;
        ModernConfig.blockEntityCull = true;
        ModernConfig.entityAnimCull = true;
        ModernConfig.armorStandCull = true;
        ModernConfig.itemFrameCull = true;
    }

    private static void applyCasualModFlags() {
        PerformanceConfig.boostFps = true;
        PerformanceConfig.applyVanillaPreset = true;
        PerformanceConfig.oldAnimations = true;
        if (PerformanceConfig.particleMode == PerformanceConfig.ParticleMode.MINIMAL) {
            PerformanceConfig.particleMode = PerformanceConfig.ParticleMode.REDUCED;
        }
        ModernConfig.oldAnimations = true;
        // Casual deja FOV dinamico como esté el config del user si lo tocó;
        // default al primer style apply lo pone on.
        ModernConfig.shaderAutoOffInMatch = true;
    }

    private static void applyCompetitiveOptions(GameOptions options) {
        // Fluidez: sin “suavizado” que traga input y FOV.
        options.getBobView().setValue(false);
        options.getAttackIndicator().setValue(AttackIndicator.OFF);
        if (options.getFovEffectScale() != null) {
            options.getFovEffectScale().setValue(0.0);
        }
        if (options.getDistortionEffectScale() != null) {
            options.getDistortionEffectScale().setValue(0.0);
        }
        if (options.getDamageTiltStrength() != null) {
            options.getDamageTiltStrength().setValue(0.0);
        }
        options.getEnableVsync().setValue(false);
        int max = options.getMaxFps().getValue();
        if (max > 0 && max < 240) {
            options.getMaxFps().setValue(260);
        }
        options.getCloudRenderMode().setValue(CloudRenderMode.OFF);
        options.getPreset().setValue(GraphicsMode.FAST);
        options.getEntityShadows().setValue(false);
        options.getParticles().setValue(ParticlesMode.MINIMAL);
        options.getViewDistance().setValue(clampChunks(PerformanceConfig.renderDistance));
        options.getSimulationDistance().setValue(clampChunks(PerformanceConfig.simulationDistance));
        options.getEntityDistanceScaling().setValue(
            Math.max(0.25, Math.min(1.0, PerformanceConfig.entityDistanceScaling))
        );
    }

    private static void applyCasualOptions(GameOptions options) {
        options.getEnableVsync().setValue(false);
        options.getCloudRenderMode().setValue(CloudRenderMode.OFF);
        options.getPreset().setValue(GraphicsMode.FAST);
        options.getEntityShadows().setValue(false);
        ParticlesMode particles = switch (PerformanceConfig.particleMode) {
            case ALL -> ParticlesMode.ALL;
            case REDUCED -> ParticlesMode.DECREASED;
            case MINIMAL -> ParticlesMode.MINIMAL;
        };
        options.getParticles().setValue(particles);
        int max = options.getMaxFps().getValue();
        if (max > 0 && max < 144) {
            options.getMaxFps().setValue(180);
        }
    }

    private static int clampChunks(int value) {
        return Math.max(2, Math.min(32, value));
    }
}
