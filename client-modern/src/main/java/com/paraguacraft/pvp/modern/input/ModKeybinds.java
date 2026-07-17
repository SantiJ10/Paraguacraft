package com.paraguacraft.pvp.modern.input;

import com.paraguacraft.pvp.modern.gui.ModMenuScreen;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import org.lwjgl.glfw.GLFW;

/** Right Shift abre el menú del mod (estilo Lunar). */
public final class ModKeybinds {

    private static KeyBinding openModMenu;

    private ModKeybinds() {}

    public static void register() {
        openModMenu = KeyBindingHelper.registerKeyBinding(new KeyBinding(
            "key.paraguacraftpvp-modern.mod_menu",
            InputUtil.Type.KEYSYM,
            GLFW.GLFW_KEY_RIGHT_SHIFT,
            KeyBinding.Category.MISC
        ));
        ClientTickEvents.END_CLIENT_TICK.register(ModKeybinds::tick);
    }

    private static void tick(MinecraftClient client) {
        while (openModMenu.wasPressed()) {
            if (client.currentScreen == null || client.player != null) {
                client.setScreen(new ModMenuScreen(client.currentScreen));
            }
        }
    }
}
