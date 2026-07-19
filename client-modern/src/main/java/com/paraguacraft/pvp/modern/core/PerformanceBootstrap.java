package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.LauncherProfile;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientLifecycleEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.CloudRenderMode;
import net.minecraft.client.option.GameOptions;
import net.minecraft.client.option.GraphicsMode;
import net.minecraft.particle.ParticlesMode;

import java.nio.file.Path;

/** Aplica preset PvP al arrancar; respeta tier de hardware del launcher. */
public final class PerformanceBootstrap {

    private PerformanceBootstrap() {}

    public static void register() {
        ClientLifecycleEvents.CLIENT_STARTED.register(PerformanceBootstrap::onClientStarted);
        ClientPlayConnectionEvents.JOIN.register((handler, sender, client) -> onWorldJoin(client));
    }

    private static void onClientStarted(MinecraftClient client) {
        Path gameDir = net.fabricmc.loader.api.FabricLoader.getInstance().getGameDir();
        Path marker = gameDir.resolve(".paraguacraft_vanilla_preset_applied");
        if (PerformanceConfig.boostFps && PerformanceConfig.applyVanillaPreset) {
            try {
                if (!java.nio.file.Files.isRegularFile(marker)) {
                    if (java.nio.file.Files.isRegularFile(gameDir.resolve("options.txt"))) {
                        java.nio.file.Files.writeString(marker, "existing");
                    } else {
                        applyGameOptions(client.options);
                        java.nio.file.Files.writeString(marker, "applied");
                    }
                }
            } catch (java.io.IOException ignored) {
            }
        }
        if (ModernConfig.windowedFullscreen) {
            WindowedFullscreenManager.enable(client);
        }
    }

    private static void onWorldJoin(MinecraftClient client) {
        QuickPlayState.onJoin(client);
        if (!PerformanceConfig.memoryCleanupOnWorldChange) {
            return;
        }
        client.execute(System::gc);
    }

    private static void applyGameOptions(GameOptions options) {
        String tier = LauncherProfile.hardwareTier == null ? "media" : LauncherProfile.hardwareTier.toLowerCase();
        int renderDistance = PerformanceConfig.renderDistance;
        int simDistance = PerformanceConfig.simulationDistance;
        double entityScale = PerformanceConfig.entityDistanceScaling;
        ParticlesMode particles = toParticlesMode(PerformanceConfig.particleMode);
        boolean fastGraphics = true;
        boolean ao = false;
        int biomeBlend = 1;

        if ("alta".equals(tier) || "high".equals(tier)) {
            renderDistance = Math.max(renderDistance, 12);
            simDistance = Math.max(simDistance, 10);
            entityScale = Math.max(entityScale, 0.75);
            biomeBlend = 2;
        } else if ("media".equals(tier) || "medium".equals(tier)) {
            renderDistance = Math.max(renderDistance, 12);
            simDistance = Math.max(simDistance, 10);
            entityScale = Math.max(entityScale, 0.75);
            ao = true;
            biomeBlend = 2;
        }

        options.getViewDistance().setValue(clampChunkDistance(renderDistance));
        options.getSimulationDistance().setValue(clampChunkDistance(simDistance));
        options.getParticles().setValue(particles);
        options.getCloudRenderMode().setValue(CloudRenderMode.OFF);
        options.getPreset().setValue(fastGraphics ? GraphicsMode.FAST : GraphicsMode.FANCY);
        options.getEntityShadows().setValue(false);
        options.getAo().setValue(ao);
        options.getEntityDistanceScaling().setValue(clamp(entityScale, 0.25, 1.0));
        options.getBiomeBlendRadius().setValue(biomeBlend);
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
