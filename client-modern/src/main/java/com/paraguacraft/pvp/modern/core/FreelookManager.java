package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.entity.Entity;

/** Freelook: rota solo la camara, no el cuerpo ni la direccion de movimiento (como 1.8.9). */
public final class FreelookManager {

    public static boolean active;

    public static float cameraYaw;
    public static float cameraPitch;
    public static float prevCameraYaw;
    public static float prevCameraPitch;

    /** Yaw del cuerpo congelado al activar freelook (W/A/S/D siguen esta direccion). */
    public static float bodyYaw;

    private static float savedYaw;
    private static float savedPitch;
    private static float savedPrevYaw;
    private static float savedPrevPitch;
    private static boolean cameraSwapped;

    private FreelookManager() {}

    public static void onPress(MinecraftClient client) {
        if (!ModernConfig.freelookEnabled || client.player == null) {
            return;
        }
        active = true;
        bodyYaw = client.player.getYaw();
        cameraYaw = prevCameraYaw = bodyYaw;
        cameraPitch = prevCameraPitch = client.player.getPitch();
    }

    public static void onRelease(MinecraftClient client) {
        active = false;
    }

    public static void addMouseDelta(float yaw, float pitch) {
        if (!active) {
            return;
        }
        prevCameraYaw = cameraYaw;
        prevCameraPitch = cameraPitch;
        cameraYaw += yaw;
        cameraPitch -= pitch;
        cameraPitch = Math.max(-90.0F, Math.min(90.0F, cameraPitch));
    }

    public static float movementYaw(MinecraftClient client) {
        if (active && client != null && client.player != null) {
            return bodyYaw;
        }
        return client != null && client.player != null ? client.player.getYaw() : 0.0F;
    }

    public static void applyCameraOverride(MinecraftClient client) {
        if (!active || client == null) {
            return;
        }
        Entity view = client.getCameraEntity();
        if (view == null) {
            return;
        }
        savedYaw = view.getYaw();
        savedPitch = view.getPitch();
        savedPrevYaw = view.lastYaw;
        savedPrevPitch = view.lastPitch;
        view.setYaw(cameraYaw);
        view.setPitch(cameraPitch);
        view.lastYaw = prevCameraYaw;
        view.lastPitch = prevCameraPitch;
        cameraSwapped = true;
    }

    public static void restoreCameraOverride(MinecraftClient client) {
        if (!cameraSwapped || client == null) {
            return;
        }
        Entity view = client.getCameraEntity();
        if (view != null) {
            view.setYaw(savedYaw);
            view.setPitch(savedPitch);
            view.lastYaw = savedPrevYaw;
            view.lastPitch = savedPrevPitch;
        }
        cameraSwapped = false;
    }
}
