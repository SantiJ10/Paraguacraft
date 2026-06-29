package com.paraguacraft.pvp.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.entity.Entity;

/**
 * Freelook / Free Lock — gira SOLO la camara, sin mover el cuerpo del jugador.
 *
 * No baneable: el cuerpo nunca rota (el servidor recibe la rotacion real
 * congelada) y los raytrace/interacciones siguen usando la rotacion real,
 * porque la camara solo se sobreescribe dentro de orientCamera al renderizar.
 */
public final class FreelookManager {

    public static boolean active = false;

    // Angulos de camara del tick actual y del anterior (para interpolar al renderizar).
    public static float cameraYaw;
    public static float cameraPitch;
    public static float prevCameraYaw;
    public static float prevCameraPitch;

    // Estado real del jugador que sobreescribimos temporalmente en orientCamera.
    private static float savedYaw;
    private static float savedPitch;
    private static float savedPrevYaw;
    private static float savedPrevPitch;
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
        cameraYaw = prevCameraYaw = mc.thePlayer.rotationYaw;
        cameraPitch = prevCameraPitch = mc.thePlayer.rotationPitch;
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
     * Absorbe los deltas crudos del mouse (los mismos que recibiria
     * {@code Entity.setAngles}) aplicando el factor 0.15 de vanilla, de modo que
     * la sensibilidad sea identica a la del juego. Se llama una vez por tick.
     */
    public static void addMouseDelta(float yaw, float pitch) {
        if (!active) {
            return;
        }
        prevCameraYaw = cameraYaw;
        prevCameraPitch = cameraPitch;
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
        savedPrevYaw = view.prevRotationYaw;
        savedPrevPitch = view.prevRotationPitch;
        // orientCamera interpola entre prevRotation y rotation con partialTicks,
        // asi la camara se ve fluida aunque los angulos se actualicen a 20 Hz.
        view.rotationYaw = cameraYaw;
        view.rotationPitch = cameraPitch;
        view.prevRotationYaw = prevCameraYaw;
        view.prevRotationPitch = prevCameraPitch;
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
            view.prevRotationYaw = savedPrevYaw;
            view.prevRotationPitch = savedPrevPitch;
        }
        cameraSwapped = false;
    }
}
