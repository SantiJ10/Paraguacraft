package com.paraguacraft.pvp.modern.hud;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.minecraft.client.MinecraftClient;
import org.lwjgl.glfw.GLFW;

/** CPS en tick (no en render) para evitar micro-stutters. */
public final class HudCpsTracker {

    private static long lastLeftClick;
    private static long lastRightClick;
    private static int leftCps;
    private static int rightCps;
    private static int leftTicks;
    private static int rightTicks;
    private static long cpsWindowStart;

    private HudCpsTracker() {}

    public static void register() {
        ClientTickEvents.END_CLIENT_TICK.register(HudCpsTracker::tick);
    }

    public static int leftCps() {
        return leftCps;
    }

    public static int rightCps() {
        return rightCps;
    }

    private static void tick(MinecraftClient client) {
        if (!ModernConfig.showCps || client == null || client.player == null) {
            return;
        }
        long now = System.currentTimeMillis();
        if (now - cpsWindowStart > 1000L) {
            leftCps = leftTicks;
            rightCps = rightTicks;
            leftTicks = 0;
            rightTicks = 0;
            cpsWindowStart = now;
        }
        long window = client.getWindow().getHandle();
        boolean left = GLFW.glfwGetMouseButton(window, GLFW.GLFW_MOUSE_BUTTON_LEFT) == GLFW.GLFW_PRESS;
        boolean right = GLFW.glfwGetMouseButton(window, GLFW.GLFW_MOUSE_BUTTON_RIGHT) == GLFW.GLFW_PRESS;
        if (left && now - lastLeftClick > 50L) {
            lastLeftClick = now;
            leftTicks++;
        }
        if (right && now - lastRightClick > 50L) {
            lastRightClick = now;
            rightTicks++;
        }
    }
}
