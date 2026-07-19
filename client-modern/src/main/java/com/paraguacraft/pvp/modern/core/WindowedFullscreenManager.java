package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.util.Window;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.glfw.GLFWVidMode;
import org.lwjgl.system.MemoryStack;

import java.nio.IntBuffer;

/**
 * Pantalla completa en ventana (borderless) estilo Patcher/Lunar — LWJGL3 + Java 21.
 * Sin fullscreen exclusivo: alt-tab instantáneo y captura por ventana.
 */
public final class WindowedFullscreenManager {

    private static int savedX;
    private static int savedY;
    private static int savedW;
    private static int savedH;
    private static boolean active;

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
        GLFW.glfwSetWindowAttrib(handle, GLFW.GLFW_DECORATED, GLFW.GLFW_FALSE);
        long monitor = GLFW.glfwGetPrimaryMonitor();
        GLFWVidMode mode = monitor != 0 ? GLFW.glfwGetVideoMode(monitor) : null;
        if (mode != null) {
            GLFW.glfwSetWindowMonitor(handle, 0, 0, 0, mode.width(), mode.height(), mode.refreshRate());
        }
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

    /** Reaplica borderless si el mod está ON y la ventana perdió el estado (resize, alt-tab). */
    public static void heal(MinecraftClient client) {
        if (!active || client == null) {
            return;
        }
        long handle = client.getWindow().getHandle();
        if (GLFW.glfwGetWindowAttrib(handle, GLFW.GLFW_DECORATED) != 0) {
            GLFW.glfwSetWindowAttrib(handle, GLFW.GLFW_DECORATED, GLFW.GLFW_FALSE);
            long monitor = GLFW.glfwGetPrimaryMonitor();
            GLFWVidMode mode = monitor != 0 ? GLFW.glfwGetVideoMode(monitor) : null;
            if (mode != null) {
                GLFW.glfwSetWindowMonitor(handle, 0, 0, 0, mode.width(), mode.height(), mode.refreshRate());
            }
        }
    }
}
