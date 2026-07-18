package com.paraguacraft.pvp.modern.util;

import com.paraguacraft.pvp.modern.gui.SkinChangerScreen;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.Screen;

/** Abre el Skin Changer propio (no la pantalla vanilla de capas). */
public final class SkinHelper {

    private SkinHelper() {}

    public static void open(Screen parent, MinecraftClient client) {
        client.setScreen(new SkinChangerScreen(parent));
    }
}
