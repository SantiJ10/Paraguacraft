package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.util.Window;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.glfw.GLFWVidMode;
import org.lwjgl.system.MemoryStack;

import java.nio.IntBuffer;

/**
 * Borderless (ventana sin bordes a tamaño de monitor) compatible con captura/overlay de Discord.
 * <p>
 * Importante: no reaplicar {@code glfwSetWindowMonitor} en cada tick — Discord Overlay
 * toca el estilo de la ventana y un "heal" agresivo pelea con él (ventana rota / sale del juego).
 */
public final class WindowedFullscreenManager {

    private static int savedX;
    private static int savedY;
    private static int savedW;
    private static int savedH;
    private static boolean active;
    private static long lastHealMs;

    private WindowedFullscreenManager() {}

    public static boolean isActive() {
        return active;
    }

    public static void toggle(MinecraftClient client) {
        if (client == null) {
            return;
        }
        if (active) {
            disable(client);
        } else {
            enable(client);
        }
    }

    public static void enable(MinecraftClient client) {
        if (client == null || active) {
            return;
        }
        applyBorderless(client);
    }

    public static void disable(MinecraftClient client) {
        if (client == null || !active) {
            return;
        }
        restoreWindowed(client);
    }

    public static void sync(MinecraftClient client) {
        if (client == null) {
            return;
        }
        if (ModernConfig.windowedFullscreen) {
            enable(client);
        } else {
            disable(client);
        }
    }

    private static void applyBorderless(MinecraftClient client) {
        Window window = client.getWindow();
        long handle = window.getHandle();
        try (MemoryStack stack = MemoryStack.stackPush()) {
            IntBuffer x = stack.mallocInt(1);
            IntBuffer y = stack.mallocInt(1);
            GLFW.glfwGetWindowPos(handle, x, y);
            savedX = x.get(0);
            savedY = y.get(0);
        }
        savedW = Math.max(window.getWidth(), 854);
        savedH = Math.max(window.getHeight(), 480);

        long monitor = GLFW.glfwGetPrimaryMonitor();
        GLFWVidMode mode = monitor != 0 ? GLFW.glfwGetVideoMode(monitor) : null;
        int monX = 0;
        int monY = 0;
        int monW = mode != null ? mode.width() : savedW;
        int monH = mode != null ? mode.height() : savedH;
        if (monitor != 0) {
            try (MemoryStack stack = MemoryStack.stackPush()) {
                IntBuffer mx = stack.mallocInt(1);
                IntBuffer my = stack.mallocInt(1);
                GLFW.glfwGetMonitorPos(monitor, mx, my);
                monX = mx.get(0);
                monY = my.get(0);
            }
        }

        // Ventana normal sin decoración (NO fullscreen exclusivo): Discord la lista para compartir.
        GLFW.glfwSetWindowAttrib(handle, GLFW.GLFW_AUTO_ICONIFY, GLFW.GLFW_FALSE);
        GLFW.glfwSetWindowAttrib(handle, GLFW.GLFW_DECORATED, GLFW.GLFW_FALSE);
        // Evitar glfwSetWindowMonitor: deja el HWND más "estable" para overlay/captura.
        GLFW.glfwSetWindowMonitor(handle, 0, monX, monY, monW, monH, GLFW.GLFW_DONT_CARE);
        GLFW.glfwSetWindowPos(handle, monX, monY);
        GLFW.glfwSetWindowSize(handle, monW, monH);
        active = true;
        client.onResolutionChanged();
    }

    private static void restoreWindowed(MinecraftClient client) {
        Window window = client.getWindow();
        long handle = window.getHandle();
        GLFW.glfwSetWindowAttrib(handle, GLFW.GLFW_DECORATED, GLFW.GLFW_TRUE);
        GLFW.glfwSetWindowMonitor(handle, 0, savedX, savedY, savedW, savedH, GLFW.GLFW_DONT_CARE);
        active = false;
        client.onResolutionChanged();
    }

    /**
     * Heal suave: solo restaura decoración si se perdió, con cooldown.
     * No pelear con Discord Overlay (que puede tocar el estilo temporalmente).
     */
    public static void heal(MinecraftClient client) {
        if (!active || !ModernConfig.windowedFullscreen || client == null) {
            return;
        }
        long now = System.currentTimeMillis();
        if (now - lastHealMs < 2500L) {
            return;
        }
        long handle = client.getWindow().getHandle();
        if (GLFW.glfwGetWindowAttrib(handle, GLFW.GLFW_DECORATED) == 0) {
            return;
        }
        // Solo quitar bordes; no redimensionar/monitor (rompe overlay).
        lastHealMs = now;
        GLFW.glfwSetWindowAttrib(handle, GLFW.GLFW_DECORATED, GLFW.GLFW_FALSE);
        GLFW.glfwSetWindowAttrib(handle, GLFW.GLFW_AUTO_ICONIFY, GLFW.GLFW_FALSE);
    }
}
