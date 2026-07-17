package com.paraguacraft.pvp.modern.util;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.option.OptionsScreen;
import net.minecraft.text.Text;
import net.fabricmc.loader.api.FabricLoader;

/** Abre Mod Menu (Fabric) si está instalado. */
public final class FabricSettingsHelper {

    private FabricSettingsHelper() {}

    public static void open(Screen parent, MinecraftClient client) {
        if (FabricLoader.getInstance().isModLoaded("modmenu")) {
            try {
                Class<?> modMenu = Class.forName("com.terraformersmc.modmenu.gui.ModsScreen");
                Object screen = modMenu.getConstructor(Screen.class).newInstance(parent);
                client.setScreen((Screen) screen);
                return;
            } catch (ReflectiveOperationException ignored) {
            }
        }
        client.setScreen(new OptionsScreen(parent, client.options));
        if (client.player != null) {
            client.player.sendMessage(Text.literal("Instalá Mod Menu para configurar mods Fabric."), false);
        }
    }
}
