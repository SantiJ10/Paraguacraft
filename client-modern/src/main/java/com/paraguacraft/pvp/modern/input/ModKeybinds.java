package com.paraguacraft.pvp.modern.input;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.FreelookManager;
import com.paraguacraft.pvp.modern.gui.GuiEditHudScreen;
import com.paraguacraft.pvp.modern.gui.HypixelQuickPlayScreen;
import com.paraguacraft.pvp.modern.gui.NickFinderScreen;
import com.paraguacraft.pvp.modern.gui.ModMenuScreen;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import org.lwjgl.glfw.GLFW;

/** Atajos estilo Paraguacraft 1.8.9. */
public final class ModKeybinds {

    private static KeyBinding openModMenu;
    private static KeyBinding editHud;
    private static KeyBinding freelook;
    private static KeyBinding fullbright;
    private static KeyBinding quickPlay;
    private static KeyBinding nickFinder;
    private static KeyBinding toggleSprint;

    private ModKeybinds() {}

    public static void register() {
        openModMenu = bind("mod_menu", GLFW.GLFW_KEY_RIGHT_SHIFT);
        editHud = bind("edit_hud", GLFW.GLFW_KEY_RIGHT_CONTROL);
        freelook = bind("freelook", GLFW.GLFW_KEY_LEFT_ALT);
        if (!FabricLoader.getInstance().isModLoaded("gammautils")) {
            fullbright = bind("fullbright", GLFW.GLFW_KEY_G);
        }
        quickPlay = bind("quick_play", GLFW.GLFW_KEY_GRAVE_ACCENT);
        nickFinder = bind("nick_finder", GLFW.GLFW_KEY_N);
        toggleSprint = bind("toggle_sprint", GLFW.GLFW_KEY_M);
        ClientTickEvents.END_CLIENT_TICK.register(ModKeybinds::tick);
    }

    private static KeyBinding bind(String id, int key) {
        return KeyBindingHelper.registerKeyBinding(new KeyBinding(
            "key.paraguacraftpvp-modern." + id,
            InputUtil.Type.KEYSYM,
            key,
            KeyBinding.Category.MISC
        ));
    }

    private static void tick(MinecraftClient client) {
        while (openModMenu.wasPressed()) {
            client.setScreen(new ModMenuScreen(client.currentScreen));
        }
        while (editHud.wasPressed()) {
            client.setScreen(new GuiEditHudScreen(client.currentScreen));
        }
        while (fullbright != null && fullbright.wasPressed()) {
            ModernConfig.fullbright = !ModernConfig.fullbright;
            if (!ModernConfig.fullbright) {
                client.options.getGamma().setValue(1.0);
            }
            ModernConfig.save();
        }
        while (quickPlay.wasPressed()) {
            client.setScreen(new HypixelQuickPlayScreen(client.currentScreen));
        }
        while (nickFinder.wasPressed()) {
            if (ModernConfig.nickFinderEnabled) {
                client.setScreen(new NickFinderScreen(client.currentScreen));
            }
        }
        while (toggleSprint.wasPressed()) {
            ModernConfig.toggleSprint = !ModernConfig.toggleSprint;
            ModernConfig.save();
            if (client.player != null) {
                client.player.sendMessage(
                    net.minecraft.text.Text.literal("Toggle sprint (M): " + (ModernConfig.toggleSprint ? "ON" : "OFF")),
                    true
                );
            }
        }
        if (ModernConfig.freelookEnabled) {
            if (freelook.isPressed()) {
                if (!FreelookManager.active) {
                    FreelookManager.onPress(client);
                }
            } else if (FreelookManager.active) {
                FreelookManager.onRelease(client);
            }
        }
    }
}
