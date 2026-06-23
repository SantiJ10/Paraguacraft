package com.paraguacraft.pvp.mixins;

import org.lwjgl.opengl.Display;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

/**
 * Cambia el titulo de la ventana de "Minecraft X.X.X" a "Paraguacraft X.X.X".
 */
@Mixin(value = Display.class, remap = false)
public class MixinDisplayTitle {

    @ModifyVariable(method = "setTitle", at = @At("HEAD"), argsOnly = true, remap = false)
    private static String paraguacraftTitle(String title) {
        if (title != null && title.contains("Minecraft") && !title.contains("Paraguacraft")) {
            return title.replace("Minecraft", "Paraguacraft");
        }
        return title;
    }
}
