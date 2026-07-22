package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.LauncherProfile;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.CullHelper;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientLifecycleEvents;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.CloudRenderMode;
import net.minecraft.client.option.GameOptions;
import net.minecraft.client.option.GraphicsMode;
import net.minecraft.particle.ParticlesMode;
import org.lwjgl.glfw.GLFW;

import java.nio.file.Path;

/** Aplica preset PvP al arrancar; respeta tier de hardware del launcher. */
public final class PerformanceBootstrap {

    private static Integer savedMaxFps;
    private static boolean idleFpsActive;

    private PerformanceBootstrap() {}

    public static void register() {
        ClientLifecycleEvents.CLIENT_STARTED.register(PerformanceBootstrap::onClientStarted);
        ClientLifecycleEvents.CLIENT_STOPPING.register(PerformanceBootstrap::restoreMaxFps);
        ClientPlayConnectionEvents.JOIN.register((handler, sender, client) -> onWorldJoin(client));
        ClientTickEvents.END_CLIENT_TICK.register(PerformanceBootstrap::tickIdleFps);
    }

    /**
     * Idle FPS (paridad 1.8.9): baja el limite de FPS cuando la ventana no
     * tiene foco (minimizada/alt-tab) para no gastar CPU/GPU de fondo, y lo
     * restaura al volver (cambio transitorio, sin tocar options.txt).
     *
     * <p>Cull real (entity/nametag/block entity/anim/decor) vive en mixins
     * performance y {@link CullHelper}.
     */
    private static void tickIdleFps(MinecraftClient client) {
        if (!PerformanceConfig.reduceFpsWhenMinimized) {
            if (idleFpsActive) {
                restoreMaxFps(client);
            }
            return;
        }
        boolean focused = client.getWindow() != null
            && GLFW.glfwGetWindowAttrib(client.getWindow().getHandle(), GLFW.GLFW_FOCUSED) == GLFW.GLFW_TRUE;
        if (!focused && !idleFpsActive) {
            savedMaxFps = client.options.getMaxFps().getValue();
            client.options.getMaxFps().setValue(Math.max(1, PerformanceConfig.minimizedFps));
            idleFpsActive = true;
        } else if (focused && idleFpsActive) {
            restoreMaxFps(client);
        }
    }

    private static void restoreMaxFps(MinecraftClient client) {
        if (idleFpsActive && savedMaxFps != null) {
            client.options.getMaxFps().setValue(savedMaxFps);
        }
        idleFpsActive = false;
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

    /** Boton "Aplicar preset de hardware" del Mod Menu: re-ejecuta el auto-preset a demanda, sin esperar al marker de arranque. */
    public static void applyPresetNow(MinecraftClient client) {
        applyGameOptions(client.options);
    }

    /** Boton "Particulas" del Mod Menu: aplica el modo elegido de inmediato (sin esperar a reiniciar). */
    public static void applyParticleModeNow(MinecraftClient client) {
        client.options.getParticles().setValue(toParticlesMode(PerformanceConfig.particleMode));
    }

    /** Boton "Limpiar memoria" del Mod Menu: misma rutina que ya corre sola al cambiar de mundo, pero manual. */
    public static void cleanMemoryNow() {
        System.gc();
    }

    private static void onWorldJoin(MinecraftClient client) {
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
