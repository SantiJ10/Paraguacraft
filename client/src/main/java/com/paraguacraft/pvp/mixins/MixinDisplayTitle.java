package com.paraguacraft.pvp.mixins;

import org.lwjgl.opengl.Display;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

/**
 * Cambia el titulo de la ventana de "Minecraft X.X.X" a "Paraguacraft PvP".
 */
@Mixin(value = Display.class, remap = false)
public class MixinDisplayTitle {

    private static final String WINDOW_TITLE = "Paraguacraft PvP";

    private static String paraguacraftTitle(String title) {
        if (title == null || title.contains("Minecraft") || title.contains("Paraguacraft")) {
            return WINDOW_TITLE;
        }
        return title;
    }

    @ModifyVariable(method = "setTitle", at = @At("HEAD"), argsOnly = true, remap = false)
    private static String paraguacraftTitleVar(String title) {
        return paraguacraftTitle(title);
    }
}
