package com.paraguacraft.pvp.mixins;

import org.lwjgl.opengl.Display;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

/**
 * Discord Overlay exige que el título empiece por {@code Minecraft}.
 * Marca: {@code Minecraft - Paraguacraft [1.8.9/PvP]}.
 */
@Mixin(value = Display.class, remap = false)
public class MixinDisplayTitle {

    private static final String WINDOW_TITLE = "Minecraft - Paraguacraft [1.8.9/PvP]";

    private static String paraguacraftTitle(String title) {
        if (title == null || title.isEmpty()) {
            return WINDOW_TITLE;
        }
        if (title.startsWith("Minecraft - Paraguacraft")) {
            return title;
        }
        if (title.contains("Minecraft") || title.contains("Paraguacraft")) {
            return WINDOW_TITLE;
        }
        return title;
    }

    @ModifyVariable(method = "setTitle", at = @At("HEAD"), argsOnly = true, remap = false)
    private static String paraguacraftTitleVar(String title) {
        return paraguacraftTitle(title);
    }
}
