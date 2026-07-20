package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.mixin.EntityRotationAccessor;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.Perspective;
import net.minecraft.entity.Entity;

/** Freelook estilo 1.8.9: camara libre en 3ª persona, cuerpo congelado. */
public final class FreelookManager {

    public static boolean active;

    public static float cameraYaw;
    public static float cameraPitch;
    public static float prevCameraYaw;
    public static float prevCameraPitch;

    /** Yaw del cuerpo congelado al activar freelook (W/A/S/D siguen esta direccion). */
    public static float bodyYaw;

    private static Perspective savedPerspective = Perspective.FIRST_PERSON;
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
        savedPerspective = client.options.getPerspective();
        client.options.setPerspective(Perspective.THIRD_PERSON_BACK);
    }

    public static void onRelease(MinecraftClient client) {
        if (!active) {
            return;
        }
        active = false;
        if (client != null) {
            client.options.setPerspective(savedPerspective);
        }
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

    /** Intercambia rotacion de la entidad durante Camera.update (como orientCamera en 1.8.9). */
    public static void applyCameraOverride(Entity entity) {
        if (!active || entity == null) {
            return;
        }
        EntityRotationAccessor rot = (EntityRotationAccessor) entity;
        savedYaw = entity.getYaw();
        savedPitch = entity.getPitch();
        savedPrevYaw = rot.paraguacraft$getLastYaw();
        savedPrevPitch = rot.paraguacraft$getLastPitch();
        entity.setYaw(cameraYaw);
        entity.setPitch(cameraPitch);
        rot.paraguacraft$setLastYaw(prevCameraYaw);
        rot.paraguacraft$setLastPitch(prevCameraPitch);
        cameraSwapped = true;
    }

    public static void restoreCameraOverride(Entity entity) {
        if (!cameraSwapped || entity == null) {
            return;
        }
        EntityRotationAccessor rot = (EntityRotationAccessor) entity;
        entity.setYaw(savedYaw);
        entity.setPitch(savedPitch);
        rot.paraguacraft$setLastYaw(savedPrevYaw);
        rot.paraguacraft$setLastPitch(savedPrevPitch);
        cameraSwapped = false;
    }

    public static float renderYaw(float tickDelta) {
        return prevCameraYaw + (cameraYaw - prevCameraYaw) * tickDelta;
    }

    public static float renderPitch(float tickDelta) {
        return prevCameraPitch + (cameraPitch - prevCameraPitch) * tickDelta;
    }
}
