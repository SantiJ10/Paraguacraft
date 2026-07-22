package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;

/** Un solo punto para brillo: Gamma Utils si esta cargado, sino fullbright interno. */
public final class FullbrightManager {

    private FullbrightManager() {}

    public static boolean isActive() {
        if (GammaUtilsBootstrap.isLoaded()) {
            return GammaUtilsBootstrap.isEnabled();
        }
        return ModernConfig.fullbright;
    }

    public static void toggle(MinecraftClient client) {
        if (GammaUtilsBootstrap.isLoaded()) {
            GammaUtilsBootstrap.toggleFullbright(client);
            ModernConfig.fullbright = GammaUtilsBootstrap.isEnabled();
        } else {
            ModernConfig.fullbright = !ModernConfig.fullbright;
            if (!ModernConfig.fullbright && client != null) {
                client.options.getGamma().setValue(1.0);
            }
        }
        ModernConfig.save();
    }

    public static String menuLabel() {
        String state = isActive() ? "ON" : "OFF";
        if (GammaUtilsBootstrap.isLoaded()) {
            return "Brillo (G): " + state + " · Gamma Utils";
        }
        return "Brillo (G): " + state;
    }

    public static String backendHint() {
        return GammaUtilsBootstrap.isLoaded()
            ? "Usa el mod Gamma Utils (tecla G)"
            : "Fullbright integrado del cliente";
    }
}
