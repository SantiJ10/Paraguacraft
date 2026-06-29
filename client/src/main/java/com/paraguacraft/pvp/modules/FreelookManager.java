package com.paraguacraft.pvp.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.entity.Entity;

/** Freelook / Free Lock — gira la cámara sin mover al jugador. */
public final class FreelookManager {

    public static boolean active = false;
    public static float cameraYaw;
    public static float cameraPitch;

    private static float savedYaw;
    private static float savedPitch;
    private static boolean cameraSwapped = false;
    private static int savedPerspective;

    private FreelookManager() {}

    public static void onPress() {
        if (!ModConfig.freelookEnabled) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null) {
            return;
        }
        active = true;
        cameraYaw = mc.thePlayer.rotationYaw;
        cameraPitch = mc.thePlayer.rotationPitch;
        savedPerspective = mc.gameSettings.thirdPersonView;
        mc.gameSettings.thirdPersonView = 1;
    }

    public static void onRelease() {
        if (!active) {
            return;
        }
        active = false;
        Minecraft.getMinecraft().gameSettings.thirdPersonView = savedPerspective;
    }

    /**
     * Absorbe los deltas crudos del mouse (los mismos que recibiría
     * {@code EntityPlayerSP.setAngles}) aplicando el factor 0.15 de vanilla.
     * Así la sensibilidad es idéntica a la del juego, sin giros extremos.
     */
    public static void addMouseDelta(float yaw, float pitch) {
        if (!active) {
            return;
        }
        cameraYaw += yaw * 0.15F;
        cameraPitch -= pitch * 0.15F;
        if (cameraPitch > 90.0F) {
            cameraPitch = 90.0F;
        }
        if (cameraPitch < -90.0F) {
            cameraPitch = -90.0F;
        }
    }

    public static void applyCameraOverride(Minecraft mc) {
        if (!active || mc == null) {
            return;
        }
        Entity view = mc.getRenderViewEntity();
        if (view == null) {
            return;
        }
        savedYaw = view.rotationYaw;
        savedPitch = view.rotationPitch;
        view.rotationYaw = cameraYaw;
        view.rotationPitch = cameraPitch;
        view.prevRotationYaw = cameraYaw;
        view.prevRotationPitch = cameraPitch;
        cameraSwapped = true;
    }

    public static void restoreCameraOverride(Minecraft mc) {
        if (!cameraSwapped || mc == null) {
            return;
        }
        Entity view = mc.getRenderViewEntity();
        if (view != null) {
            view.rotationYaw = savedYaw;
            view.rotationPitch = savedPitch;
            view.prevRotationYaw = savedYaw;
            view.prevRotationPitch = savedPitch;
        }
        cameraSwapped = false;
    }
}
