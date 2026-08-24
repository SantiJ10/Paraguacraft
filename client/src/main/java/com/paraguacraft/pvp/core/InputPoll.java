package com.paraguacraft.pvp.core;

import net.minecraft.client.settings.KeyBinding;
import org.lwjgl.input.Keyboard;
import org.lwjgl.input.Mouse;

/**
 * Una sola lectura de ratón por frame HUD + nombres de tecla cacheados.
 * Evita {@code Keyboard.getKeyName} y {@code Mouse.isButtonDown} repetidos.
 */
public final class InputPoll {

    private static final String[] NAMES = new String[256];
    public static boolean lmb;
    public static boolean rmb;

    private InputPoll() {}

    public static void beginFrame() {
        lmb = Mouse.isButtonDown(0);
        rmb = Mouse.isButtonDown(1);
    }

    public static boolean mouse(int button) {
        return button == 0 ? lmb : (button == 1 ? rmb : Mouse.isButtonDown(button));
    }

    public static String name(KeyBinding key) {
        int code = key.getKeyCode();
        if (code < 0) {
            int btn = code + 100;
            if (btn == 0) {
                return "LMB";
            }
            if (btn == 1) {
                return "RMB";
            }
            return "M" + btn;
        }
        if (code >= NAMES.length) {
            return "?";
        }
        if (NAMES[code] == null) {
            String n = Keyboard.getKeyName(code);
            NAMES[code] = n == null || n.isEmpty() ? "?" : n;
        }
        return NAMES[code];
    }
}
