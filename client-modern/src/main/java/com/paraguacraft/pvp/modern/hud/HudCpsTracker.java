package com.paraguacraft.pvp.modern.hud;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import org.lwjgl.glfw.GLFW;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

/**
 * CPS por flanco de bajada (clic real), no por frames/ticks mantenidos.
 * Ventana rodante de 1 s, igual que el cliente 1.8.9.
 */
public final class HudCpsTracker {

    private static final List<Long> leftClicks = new ArrayList<>();
    private static final List<Long> rightClicks = new ArrayList<>();
    private static boolean wasLmb;
    private static boolean wasRmb;

    private HudCpsTracker() {}

    public static void register() {
        // El muestreo es por frame desde {@link HudRenderer}, no por tick.
    }

    public static int leftCps() {
        return leftClicks.size();
    }

    public static int rightCps() {
        return rightClicks.size();
    }

    public static void poll(MinecraftClient client) {
        if (!ModernConfig.showCps && !ModernConfig.showKeystrokes) {
            return;
        }
        if (client == null || client.player == null || client.getWindow() == null) {
            return;
        }
        long now = System.currentTimeMillis();
        long handle = client.getWindow().getHandle();
        boolean lmb = GLFW.glfwGetMouseButton(handle, GLFW.GLFW_MOUSE_BUTTON_LEFT) == GLFW.GLFW_PRESS;
        boolean rmb = GLFW.glfwGetMouseButton(handle, GLFW.GLFW_MOUSE_BUTTON_RIGHT) == GLFW.GLFW_PRESS;
        if (lmb && !wasLmb) {
            leftClicks.add(now);
        }
        if (rmb && !wasRmb) {
            rightClicks.add(now);
        }
        wasLmb = lmb;
        wasRmb = rmb;
        expire(leftClicks, now);
        expire(rightClicks, now);
    }

    private static void expire(List<Long> clicks, long now) {
        Iterator<Long> it = clicks.iterator();
        while (it.hasNext()) {
            if (now - it.next() > 1000L) {
                it.remove();
            }
        }
    }
}
