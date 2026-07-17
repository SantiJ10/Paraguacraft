package com.paraguacraft.pvp.modern.util;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.option.OptionsScreen;
import net.minecraft.client.gui.screen.option.SkinOptionsScreen;

/** Abre la pantalla de skin / apariencia del jugador. */
public final class SkinHelper {

    private SkinHelper() {}

    public static void open(Screen parent, MinecraftClient client) {
        client.setScreen(new SkinOptionsScreen(parent, client.options));
    }
}
