package com.paraguacraft.pvp.modern.mixin;

import net.minecraft.client.util.Window;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

/**
 * Discord Overlay exige que el título empiece por {@code Minecraft}.
 * Vanilla 1.15+ añade "Multijugador (servidor de terceros)" al entrar a un
 * server y Discord deja de detectarlo. Forzamos la marca Paraguacraft.
 */
@Mixin(Window.class)
public class MixinWindowTitle {

    private static final String WINDOW_TITLE = "Minecraft - Paraguacraft [1.21.11/PvP]";

    @ModifyVariable(method = "setTitle", at = @At("HEAD"), argsOnly = true)
    private String paraguacraft$discordDetectableTitle(String title) {
        if (title != null && title.startsWith("Minecraft - Paraguacraft")) {
            return title;
        }
        return WINDOW_TITLE;
    }
}
