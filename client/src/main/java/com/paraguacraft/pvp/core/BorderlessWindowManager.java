package com.paraguacraft.pvp.core;

import com.paraguacraft.pvp.modules.ModConfig;
import com.sun.jna.platform.win32.User32;
import com.sun.jna.platform.win32.WinDef.HWND;
import com.sun.jna.platform.win32.WinDef.RECT;
import com.sun.jna.platform.win32.WinUser;
import org.lwjgl.opengl.Display;
import org.lwjgl.opengl.DisplayMode;

/**
 * Borderless fullscreen en caliente vía Win32 (estilo Lunar).
 * En sistemas no-Windows cae al comportamiento LWJGL básico.
 */
public final class BorderlessWindowManager {

    private static int savedWidth = 854;
    private static int savedHeight = 480;
    private static int savedX = 100;
    private static int savedY = 100;
    private static boolean saved;

    private BorderlessWindowManager() {}

    public static void applyFromConfig() {
        apply(ModConfig.borderlessWindow);
    }

    public static void apply(boolean borderless) {
        if (!Display.isCreated()) {
            return;
        }
        if (isWindows()) {
            applyWin32(borderless);
        } else {
            applyFallback(borderless);
        }
    }

    private static HWND resolveHwnd() {
        try {
            Object impl = Display.class.getMethod("getImplementation").invoke(null);
            if (impl == null) {
                return null;
            }
            java.lang.reflect.Field field = impl.getClass().getDeclaredField("hwnd");
            field.setAccessible(true);
            Object raw = field.get(impl);
            long handle;
            if (raw instanceof com.sun.jna.Pointer) {
                handle = com.sun.jna.Pointer.nativeValue((com.sun.jna.Pointer) raw);
            } else if (raw instanceof Number) {
                handle = ((Number) raw).longValue();
            } else {
                return null;
            }
            return new HWND(com.sun.jna.Pointer.createConstant(handle));
        } catch (Exception e) {
            return null;
        }
    }

    private static void applyWin32(boolean borderless) {
        try {
            HWND hwnd = resolveHwnd();
            if (hwnd.getPointer() == null) {
                applyFallback(borderless);
                return;
            }

            int style = User32.INSTANCE.GetWindowLong(hwnd, WinUser.GWL_STYLE);
            if (borderless) {
                if (!saved) {
                    savedWidth = Display.getWidth();
                    savedHeight = Display.getHeight();
                    RECT rect = new RECT();
                    User32.INSTANCE.GetWindowRect(hwnd, rect);
                    savedX = rect.left;
                    savedY = rect.top;
                    saved = true;
                }

                style &= ~(WinUser.WS_CAPTION | WinUser.WS_THICKFRAME | WinUser.WS_SYSMENU);
                User32.INSTANCE.SetWindowLong(hwnd, WinUser.GWL_STYLE, style);

                DisplayMode desktop = Display.getDesktopDisplayMode();
                User32.INSTANCE.SetWindowPos(
                    hwnd,
                    null,
                    0,
                    0,
                    desktop.getWidth(),
                    desktop.getHeight(),
                    WinUser.SWP_FRAMECHANGED | WinUser.SWP_SHOWWINDOW
                );
            } else {
                style |= WinUser.WS_CAPTION | WinUser.WS_THICKFRAME
                    | WinUser.WS_MINIMIZEBOX | WinUser.WS_MAXIMIZEBOX | WinUser.WS_SYSMENU;
                User32.INSTANCE.SetWindowLong(hwnd, WinUser.GWL_STYLE, style);

                int w = savedWidth > 0 ? savedWidth : 854;
                int h = savedHeight > 0 ? savedHeight : 480;
                Display.setDisplayMode(new DisplayMode(w, h));
                User32.INSTANCE.SetWindowPos(
                    hwnd,
                    null,
                    savedX,
                    savedY,
                    w,
                    h,
                    WinUser.SWP_FRAMECHANGED | WinUser.SWP_SHOWWINDOW
                );
                Display.setLocation(savedX, savedY);
                Display.setResizable(true);
            }
        } catch (Throwable t) {
            applyFallback(borderless);
        }
    }

    private static void applyFallback(boolean borderless) {
        try {
            if (borderless) {
                if (!saved) {
                    savedWidth = Display.getWidth();
                    savedHeight = Display.getHeight();
                    saved = true;
                }
                System.setProperty("org.lwjgl.opengl.Window.undecorated", "true");
                Display.setDisplayMode(Display.getDesktopDisplayMode());
                Display.setLocation(0, 0);
            } else {
                System.setProperty("org.lwjgl.opengl.Window.undecorated", "false");
                int w = savedWidth > 0 ? savedWidth : 854;
                int h = savedHeight > 0 ? savedHeight : 480;
                Display.setDisplayMode(new DisplayMode(w, h));
                Display.setLocation(savedX, savedY);
                Display.setResizable(true);
            }
        } catch (Exception ignored) {
        }
    }

    private static boolean isWindows() {
        String os = System.getProperty("os.name", "");
        return os.toLowerCase().contains("win");
    }
}
