package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.Perspective;
import net.minecraft.entity.Entity;

/** Freelook: rota solo la camara, no el cuerpo (como 1.8.9). */
public final class FreelookManager {

    public static boolean active;

    public static float cameraYaw;
    public static float cameraPitch;
    public static float prevCameraYaw;
    public static float prevCameraPitch;

    private static float savedYaw;
    private static float savedPitch;
    private static float savedPrevYaw;
    private static float savedPrevPitch;
    private static boolean cameraSwapped;
    private static Perspective savedPerspective;

    private FreelookManager() {}

    public static void onPress(MinecraftClient client) {
        if (!ModernConfig.freelookEnabled || client.player == null) {
            return;
        }
        active = true;
        cameraYaw = prevCameraYaw = client.player.getYaw();
        cameraPitch = prevCameraPitch = client.player.getPitch();
        savedPerspective = client.options.getPerspective();
        client.options.setPerspective(Perspective.THIRD_PERSON_BACK);
    }

    public static void onRelease(MinecraftClient client) {
        if (!active) {
            return;
        }
        active = false;
        client.options.setPerspective(savedPerspective);
    }

    public static void addMouseDelta(float yaw, float pitch) {
        if (!active) {
            return;
        }
        prevCameraYaw = cameraYaw;
        prevCameraPitch = cameraPitch;
        cameraYaw += yaw * 0.15F;
        cameraPitch -= pitch * 0.15F;
        cameraPitch = Math.max(-90.0F, Math.min(90.0F, cameraPitch));
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
