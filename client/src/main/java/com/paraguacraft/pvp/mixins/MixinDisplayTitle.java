package com.paraguacraft.pvp.mixins;

import org.lwjgl.opengl.Display;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

/**
 * Discord Overlay exige el título vanilla ({@code Minecraft 1.8.9} /
 * {@code Minecraft* 1.8.9}). Forge es cliente modificado → asterisco.
 */
@Mixin(value = Display.class, remap = false)
public class MixinDisplayTitle {

    private static final String WINDOW_TITLE = "Minecraft* 1.8.9";

    private static String paraguacraftTitle(String title) {
        if (title == null || title.isEmpty()) {
            return WINDOW_TITLE;
        }
        if (title.equals(WINDOW_TITLE)) {
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
