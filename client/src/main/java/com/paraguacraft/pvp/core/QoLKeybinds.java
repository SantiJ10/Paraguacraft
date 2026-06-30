package com.paraguacraft.pvp.core;

import com.paraguacraft.pvp.modules.ModConfig;
import com.paraguacraft.pvp.modules.QoLManager;
import net.minecraft.client.settings.KeyBinding;

/** Sincroniza teclas configurables con KeyBinding de Forge. */
public final class QoLKeybinds {

    private QoLKeybinds() {}

    public static void applyFromConfig() {
        setCode(QoLManager.menuKey, ModConfig.keyMenu);
        setCode(QoLManager.editHudKey, ModConfig.keyEditHud);
        setCode(QoLManager.toggleSprintKey, ModConfig.keyToggleSprint);
        setCode(QoLManager.toggleSprintLegacyKey, ModConfig.keyToggleSprintLegacy);
        setCode(QoLManager.fullbrightKey, ModConfig.keyFullbright);
        setCode(QoLManager.freelookKey, ModConfig.keyFreelook);
        setCode(QoLManager.quickPlayKey, ModConfig.keyQuickPlay);
        KeyBinding.resetKeyBindingArrayAndHash();
    }

    private static void setCode(KeyBinding bind, int code) {
        if (bind == null) {
            return;
        }
        try {
            java.lang.reflect.Field f = KeyBinding.class.getDeclaredField("keyCode");
            f.setAccessible(true);
            f.setInt(bind, code);
        } catch (Exception ignored) {
        }
    }
}
