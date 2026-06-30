package com.paraguacraft.pvp.core;

import com.paraguacraft.pvp.core.win32.Win32Helper;
import com.paraguacraft.pvp.core.win32.Win32Helper.Kernel32;
import com.paraguacraft.pvp.core.win32.Win32Helper.MONITORINFO;
import com.paraguacraft.pvp.core.win32.Win32Helper.RECT;
import com.paraguacraft.pvp.core.win32.Win32Helper.User32;
import com.paraguacraft.pvp.core.win32.Win32Helper.WndEnumProc;
import com.paraguacraft.pvp.modules.ModConfig;
import com.sun.jna.Native;
import com.sun.jna.Pointer;
import com.sun.jna.ptr.IntByReference;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiScreen;
import org.lwjgl.opengl.Display;

import java.awt.Canvas;
import java.awt.Component;
import java.lang.reflect.Field;
import java.lang.reflect.Method;

/**
 * Borderless Win32 — solo quita caption/frame, usa monitor completo, restaura estado exacto.
 */
public final class BorderlessWindowManager {

    private static int savedWinX;
    private static int savedWinY;
    private static int savedWinW;
    private static int savedWinH;
    private static int savedClientW;
    private static int savedClientH;
    private static int savedStyle;
    private static int savedExStyle;
    private static boolean saved;
    private static boolean active;

    private static int deferTicks;
    private static int resyncTicks;

    private static Method mcResize;
    private static Method mcUpdateFramebuffer;

    private BorderlessWindowManager() {}

    public static void scheduleApplyFromConfig() {
        if (ModConfig.borderlessWindow) {
            deferTicks = 40;
        }
    }

    public static void clientTick() {
        if (deferTicks > 0) {
            deferTicks--;
            if (deferTicks == 0) {
                applyInternal(true);
            }
        }
        if (resyncTicks > 0) {
            resyncTicks--;
            if (resyncTicks == 0) {
                Pointer hwnd = resolveHwnd();
                if (hwnd != null) {
                    syncCanvas(hwnd);
                }
            }
        }
    }

    public static void applyFromConfig() {
        apply(ModConfig.borderlessWindow);
    }

    public static void apply(boolean borderless) {
        if (!Display.isCreated() || !isWindows()) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc != null) {
            mc.addScheduledTask(new Runnable() {
                @Override
                public void run() {
                    applyInternal(borderless);
                }
            });
        } else {
            applyInternal(borderless);
        }
    }

    private static void applyInternal(boolean borderless) {
        if (!Display.isCreated() || !isWindows()) {
            return;
        }
        try {
            Pointer hwnd = resolveHwnd();
            if (hwnd == null) {
                return;
            }

            if (borderless) {
                enableBorderless(hwnd);
            } else {
                disableBorderless(hwnd);
            }
            resyncTicks = 5;
        } catch (Throwable t) {
            // No reseteamos el toggle del usuario: si falla un intento (p.ej. en pleno
            // juego), conservamos la intención y reintentamos en el próximo arranque
            // (scheduleApplyFromConfig, que es más estable). Antes esto hacía que el
            // borderless "no se pudiera activar" porque volvía solo a OFF.
            System.err.println("[Paraguacraft] Borderless falló: " + t);
            active = false;
        }
    }

    private static void enableBorderless(Pointer hwnd) {
        if (!saved) {
            captureSavedState(hwnd);
        }

        // 1) Quitamos TODA la decoracion (borde, barra de titulo, marco redimensionable).
        int style = User32.INSTANCE.GetWindowLongW(hwnd, Win32Helper.GWL_STYLE);
        style &= ~(Win32Helper.WS_CAPTION | Win32Helper.WS_THICKFRAME | Win32Helper.WS_SYSMENU
            | Win32Helper.WS_MINIMIZEBOX | Win32Helper.WS_MAXIMIZEBOX
            | Win32Helper.WS_BORDER | Win32Helper.WS_DLGFRAME);
        User32.INSTANCE.SetWindowLongW(hwnd, Win32Helper.GWL_STYLE, style);

        // 2) ExStyle: que Discord la liste (APPWINDOW) y NO la trate como herramienta/topmost.
        //    Sin esto: Discord no encuentra la ventana para compartir ni engancha el overlay.
        int exStyle = User32.INSTANCE.GetWindowLongW(hwnd, Win32Helper.GWL_EXSTYLE);
        exStyle &= ~(Win32Helper.WS_EX_DLGMODALFRAME | Win32Helper.WS_EX_CLIENTEDGE
            | Win32Helper.WS_EX_STATICEDGE | Win32Helper.WS_EX_WINDOWEDGE
            | Win32Helper.WS_EX_TOPMOST | Win32Helper.WS_EX_TOOLWINDOW);
        exStyle |= Win32Helper.WS_EX_APPWINDOW;
        User32.INSTANCE.SetWindowLongW(hwnd, Win32Helper.GWL_EXSTYLE, exStyle);

        // 3) Ocupamos el monitor completo PERO 1px mas alto: evita que el DWM active el modo
        //    "fullscreen exclusivo / independent flip", que rompe el overlay y la captura de Discord.
        //    Asi la ventana sigue siendo borderless-windowed (compuesta) y Discord la engancha bien.
        RECT mon = fullMonitorRect(hwnd);
        int clientW = mon.right - mon.left;
        int clientH = (mon.bottom - mon.top) + 1;
        placeWindowForClient(hwnd, mon.left, mon.top, clientW, clientH, style, exStyle);
        syncCanvas(hwnd);
        active = true;
    }

    private static void disableBorderless(Pointer hwnd) {
        int style = saved ? savedStyle : Win32Helper.WS_OVERLAPPEDWINDOW;
        int exStyle = saved ? savedExStyle : User32.INSTANCE.GetWindowLongW(hwnd, Win32Helper.GWL_EXSTYLE);
        User32.INSTANCE.SetWindowLongW(hwnd, Win32Helper.GWL_STYLE, style);
        User32.INSTANCE.SetWindowLongW(hwnd, Win32Helper.GWL_EXSTYLE, exStyle);

        int clientW = savedClientW > 64 ? savedClientW : 854;
        int clientH = savedClientH > 64 ? savedClientH : 480;
        int x = saved ? savedWinX : 100;
        int y = saved ? savedWinY : 100;

        placeWindowForClient(hwnd, x, y, clientW, clientH, style, exStyle);

        try {
            Display.setLocation(x, y);
            Display.setResizable(true);
        } catch (Exception ignored) {
        }

        syncCanvas(hwnd);
        saved = false;
        active = false;
    }

    private static void placeWindowForClient(
        Pointer hwnd, int originX, int originY, int clientW, int clientH, int style, int exStyle
    ) {
        RECT r = new RECT();
        r.left = 0;
        r.top = 0;
        r.right = clientW;
        r.bottom = clientH;
        User32.INSTANCE.AdjustWindowRectEx(r, style, false, exStyle);
        int outerW = r.right - r.left;
        int outerH = r.bottom - r.top;
        User32.INSTANCE.SetWindowPos(
            hwnd,
            null,
            originX + r.left,
            originY + r.top,
            outerW,
            outerH,
            Win32Helper.SWP_FRAMECHANGED | Win32Helper.SWP_SHOWWINDOW | Win32Helper.SWP_NOZORDER
        );
    }

    private static void captureSavedState(Pointer hwnd) {
        savedStyle = User32.INSTANCE.GetWindowLongW(hwnd, Win32Helper.GWL_STYLE);
        savedExStyle = User32.INSTANCE.GetWindowLongW(hwnd, Win32Helper.GWL_EXSTYLE);
        RECT wr = new RECT();
        User32.INSTANCE.GetWindowRect(hwnd, wr);
        savedWinX = wr.left;
        savedWinY = wr.top;
        savedWinW = wr.right - wr.left;
        savedWinH = wr.bottom - wr.top;
        RECT cr = new RECT();
        User32.INSTANCE.GetClientRect(hwnd, cr);
        savedClientW = cr.right - cr.left;
        savedClientH = cr.bottom - cr.top;
        saved = true;
    }

    private static void syncCanvas(Pointer hwnd) {
        RECT cr = new RECT();
        User32.INSTANCE.GetClientRect(hwnd, cr);
        int w = cr.right - cr.left;
        int h = cr.bottom - cr.top;
        if (w < 64 || h < 64) {
            return;
        }

        Component parent = Display.getParent();
        if (parent instanceof Canvas) {
            Canvas canvas = (Canvas) parent;
            canvas.setSize(w, h);
            canvas.validate();
        }

        Minecraft mc = Minecraft.getMinecraft();
        if (mc != null) {
            resizeMc(mc, w, h);
        }
    }

    private static void resizeMc(Minecraft mc, int w, int h) {
        try {
            if (mcResize == null) {
                mcResize = Minecraft.class.getDeclaredMethod("resize", int.class, int.class);
                mcResize.setAccessible(true);
            }
            mcResize.invoke(mc, w, h);
            return;
        } catch (Throwable ignored) {
        }

        mc.displayWidth = w;
        mc.displayHeight = h;
        try {
            if (mcUpdateFramebuffer == null) {
                mcUpdateFramebuffer = Minecraft.class.getDeclaredMethod("updateFramebufferSize");
                mcUpdateFramebuffer.setAccessible(true);
            }
            mcUpdateFramebuffer.invoke(mc);
        } catch (Throwable ignored) {
        }
        GuiScreen screen = mc.currentScreen;
        if (screen != null) {
            screen.setWorldAndResolution(mc, w, h);
        }
    }

    private static RECT fullMonitorRect(Pointer hwnd) {
        MONITORINFO mi = new MONITORINFO();
        User32.INSTANCE.GetMonitorInfoW(
            User32.INSTANCE.MonitorFromWindow(hwnd, Win32Helper.MONITOR_DEFAULTTONEAREST),
            mi
        );
        return mi.rcMonitor;
    }

    private static Pointer resolveHwnd() {
        // En Minecraft 1.8.9 LWJGL2 crea la ventana SIN canvas AWT, así que
        // Display.getParent() suele ser null. La enumeración por proceso (EnumWindows)
        // es la vía más fiable; la dejamos primero.
        Pointer h = hwndFromProcess();
        if (h != null) {
            return h;
        }
        h = hwndFromAwt();
        if (h != null) {
            return h;
        }
        return hwndFromLwjgl();
    }

    private static Pointer hwndFromAwt() {
        try {
            Component parent = Display.getParent();
            if (parent == null) {
                return null;
            }
            Method m = Component.class.getDeclaredMethod("getPeer");
            m.setAccessible(true);
            Object peer = m.invoke(parent);
            if (peer == null) {
                return null;
            }
            Field hwndField = findField(peer.getClass(), "hwnd");
            if (hwndField == null) {
                return null;
            }
            hwndField.setAccessible(true);
            return handleFromObject(hwndField.get(peer));
        } catch (Exception ignored) {
            return null;
        }
    }

    private static Pointer hwndFromLwjgl() {
        try {
            Object impl = Display.class.getMethod("getImplementation").invoke(null);
            if (impl == null) {
                return null;
            }
            Pointer h = handleFromObject(impl);
            if (h != null) {
                return h;
            }
            Field peerField = findField(impl.getClass(), "peer_info");
            if (peerField != null) {
                peerField.setAccessible(true);
                Object peer = peerField.get(impl);
                if (peer != null) {
                    return handleFromObject(peer);
                }
            }
        } catch (Exception ignored) {
        }
        return null;
    }

    private static Pointer hwndFromProcess() {
        final int pid = Kernel32.INSTANCE.GetCurrentProcessId();
        final Pointer[] found = new Pointer[1];
        User32.INSTANCE.EnumWindows(new WndEnumProc() {
            @Override
            public boolean callback(Pointer hwnd, Pointer data) {
                IntByReference wndPid = new IntByReference();
                User32.INSTANCE.GetWindowThreadProcessId(hwnd, wndPid);
                if (wndPid.getValue() != pid || !User32.INSTANCE.IsWindowVisible(hwnd)) {
                    return true;
                }
                char[] buf = new char[512];
                User32.INSTANCE.GetWindowTextW(hwnd, buf, 512);
                String title = Native.toString(buf);
                if (title.contains("Minecraft") || title.contains("Paraguacraft")) {
                    found[0] = hwnd;
                    return false;
                }
                return true;
            }
        }, null);
        return found[0];
    }

    private static Pointer handleFromObject(Object raw) {
        if (raw == null) {
            return null;
        }
        long handle;
        if (raw instanceof Pointer) {
            handle = Pointer.nativeValue((Pointer) raw);
        } else if (raw instanceof Number) {
            handle = ((Number) raw).longValue();
        } else {
            return null;
        }
        return Win32Helper.hwnd(handle);
    }

    private static Field findField(Class<?> type, String name) {
        Class<?> c = type;
        while (c != null) {
            try {
                return c.getDeclaredField(name);
            } catch (NoSuchFieldException e) {
                c = c.getSuperclass();
            }
        }
        return null;
    }

    private static boolean isWindows() {
        return System.getProperty("os.name", "").toLowerCase().contains("win");
    }

    public static boolean isActive() {
        return active;
    }
}
