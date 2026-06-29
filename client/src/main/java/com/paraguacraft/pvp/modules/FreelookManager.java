package com.paraguacraft.pvp.modules;

import net.minecraft.client.Minecraft;
import org.lwjgl.input.Mouse;

/** Freelook / perspective — girar cámara sin mover al personaje. */
public final class FreelookManager {

    public static boolean active = false;
    public static float cameraYaw;
    public static float cameraPitch;

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
    }

    public static void onRelease() {
        active = false;
    }

    public static void updateMouse() {
        if (!active) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null || mc.currentScreen != null) {
            return;
        }
        float sens = mc.gameSettings.mouseSensitivity * 0.6F + 0.2F;
        float factor = sens * sens * sens * 8.0F * 0.15F;
        cameraYaw += Mouse.getDX() * factor;
        cameraPitch -= Mouse.getDY() * factor;
        if (cameraPitch > 90.0F) {
            cameraPitch = 90.0F;
        }
        if (cameraPitch < -90.0F) {
            cameraPitch = -90.0F;
        }
    }
}
