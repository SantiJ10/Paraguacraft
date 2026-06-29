package com.paraguacraft.pvp.core.win32;

import com.sun.jna.Callback;
import com.sun.jna.Native;
import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import com.sun.jna.ptr.IntByReference;
import com.sun.jna.win32.StdCallLibrary;
import com.sun.jna.win32.W32APIOptions;

/**
 * Win32 mínimo vía JNA core — evita jna-platform (conflicto con LWJGL/Forge 3.4.0).
 */
public final class Win32Helper {

    public static final int GWL_STYLE = -16;
    public static final int GWL_EXSTYLE = -20;

    public static final int WS_CAPTION = 0x00C00000;
    public static final int WS_THICKFRAME = 0x00040000;
    public static final int WS_SYSMENU = 0x00080000;
    public static final int WS_MINIMIZEBOX = 0x00020000;
    public static final int WS_MAXIMIZEBOX = 0x00010000;
    public static final int WS_BORDER = 0x00800000;
    public static final int WS_DLGFRAME = 0x00400000;
    public static final int WS_OVERLAPPEDWINDOW = 0x00CF0000;

    // Extended styles relevantes para que Discord liste/capture la ventana y muestre el overlay.
    public static final int WS_EX_DLGMODALFRAME = 0x00000001;
    public static final int WS_EX_TOPMOST = 0x00000008;
    public static final int WS_EX_TOOLWINDOW = 0x00000080;
    public static final int WS_EX_WINDOWEDGE = 0x00000100;
    public static final int WS_EX_CLIENTEDGE = 0x00000200;
    public static final int WS_EX_APPWINDOW = 0x00040000;
    public static final int WS_EX_STATICEDGE = 0x00020000;

    public static final int SWP_FRAMECHANGED = 0x0020;
    public static final int SWP_SHOWWINDOW = 0x0040;
    public static final int SWP_NOZORDER = 0x0004;
    public static final int SWP_NOACTIVATE = 0x0010;

    public static final int MONITOR_DEFAULTTONEAREST = 2;

    public interface User32 extends StdCallLibrary {
        User32 INSTANCE = Native.load("user32", User32.class, W32APIOptions.DEFAULT_OPTIONS);

        boolean GetClientRect(Pointer hWnd, RECT lpRect);
        boolean GetWindowRect(Pointer hWnd, RECT lpRect);
        boolean SetWindowPos(Pointer hWnd, Pointer hWndInsertAfter, int x, int y, int cx, int cy, int uFlags);
        int GetWindowLongW(Pointer hWnd, int nIndex);
        int SetWindowLongW(Pointer hWnd, int nIndex, int dwNewLong);
        boolean AdjustWindowRectEx(RECT lpRect, int dwStyle, boolean bMenu, int dwExStyle);
        boolean GetMonitorInfoW(Pointer hMonitor, MONITORINFO lpmi);
        Pointer MonitorFromWindow(Pointer hwnd, int dwFlags);
        boolean EnumWindows(WndEnumProc lpEnumFunc, Pointer lParam);
        int GetWindowThreadProcessId(Pointer hWnd, IntByReference lpdwProcessId);
        boolean IsWindowVisible(Pointer hWnd);
        int GetWindowTextW(Pointer hWnd, char[] lpString, int nMaxCount);
    }

    public interface Kernel32 extends StdCallLibrary {
        Kernel32 INSTANCE = Native.load("kernel32", Kernel32.class, W32APIOptions.DEFAULT_OPTIONS);

        int GetCurrentProcessId();
    }

    public interface WndEnumProc extends Callback {
        boolean callback(Pointer hwnd, Pointer data);
    }

    public static class RECT extends Structure {
        public int left;
        public int top;
        public int right;
        public int bottom;

        @Override
        protected java.util.List<String> getFieldOrder() {
            return java.util.Arrays.asList("left", "top", "right", "bottom");
        }
    }

    public static class MONITORINFO extends Structure {
        public int cbSize;
        public RECT rcMonitor = new RECT();
        public RECT rcWork = new RECT();
        public int dwFlags;

        public MONITORINFO() {
            cbSize = size();
        }

        @Override
        protected java.util.List<String> getFieldOrder() {
            return java.util.Arrays.asList("cbSize", "rcMonitor", "rcWork", "dwFlags");
        }
    }

    public static Pointer hwnd(long handle) {
        return handle == 0L ? null : new Pointer(handle);
    }

    public static long handle(Pointer p) {
        return p == null ? 0L : Pointer.nativeValue(p);
    }

    private Win32Helper() {}
}
