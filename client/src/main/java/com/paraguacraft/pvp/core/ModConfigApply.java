package com.paraguacraft.pvp.core;

import com.paraguacraft.pvp.modules.ModConfig;

/** Aplica opciones persistentes una vez al arrancar el cliente. */
public final class ModConfigApply {

    private static boolean applied;

    private ModConfigApply() {}

    public static void onStartup() {
        if (applied) {
            return;
        }
        applied = true;
        ModConfig.loaded = true;
        BorderlessWindowManager.scheduleApplyFromConfig();
    }
}
