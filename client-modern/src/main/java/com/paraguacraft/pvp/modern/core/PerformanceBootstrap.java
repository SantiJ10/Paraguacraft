package com.paraguacraft.pvp.modern.core;

import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientLifecycleEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.CloudRenderMode;
import net.minecraft.client.option.GameOptions;
import net.minecraft.client.option.GraphicsMode;
import net.minecraft.particle.ParticlesMode;

/** Aplica preset PvP al arrancar y libera memoria al cambiar de mundo/servidor. */
public final class PerformanceBootstrap {

    private PerformanceBootstrap() {}

    public static void register() {
        ClientLifecycleEvents.CLIENT_STARTED.register(PerformanceBootstrap::onClientStarted);
        ClientPlayConnectionEvents.JOIN.register((handler, sender, client) -> onWorldJoin(client));
    }

    private static void onClientStarted(MinecraftClient client) {
        if (!PerformanceConfig.boostFps || !PerformanceConfig.applyVanillaPreset) {
            return;
        }
        applyGameOptions(client.options);
    }

    private static void onWorldJoin(MinecraftClient client) {
        if (!PerformanceConfig.memoryCleanupOnWorldChange) {
            return;
        }
        client.execute(System::gc);
    }

    private static void applyGameOptions(GameOptions options) {
        options.getViewDistance().setValue(clampChunkDistance(PerformanceConfig.renderDistance));
        options.getSimulationDistance().setValue(clampChunkDistance(PerformanceConfig.simulationDistance));
        options.getParticles().setValue(toParticlesMode(PerformanceConfig.particleMode));
        options.getCloudRenderMode().setValue(CloudRenderMode.OFF);
        options.getPreset().setValue(GraphicsMode.FAST);
        options.getEntityShadows().setValue(false);
        options.getAo().setValue(false);
        options.getEntityDistanceScaling().setValue(
            clamp(PerformanceConfig.entityDistanceScaling, 0.25, 1.0)
        );
        options.getBiomeBlendRadius().setValue(Math.min(options.getBiomeBlendRadius().getValue(), 2));
    }

    private static ParticlesMode toParticlesMode(PerformanceConfig.ParticleMode mode) {
        return switch (mode) {
            case ALL -> ParticlesMode.ALL;
            case REDUCED -> ParticlesMode.DECREASED;
            case MINIMAL -> ParticlesMode.MINIMAL;
        };
    }

    private static int clampChunkDistance(int value) {
        return Math.max(2, Math.min(32, value));
    }

    private static double clamp(double value, double min, double max) {
        return Math.max(min, Math.min(max, value));
    }
}
