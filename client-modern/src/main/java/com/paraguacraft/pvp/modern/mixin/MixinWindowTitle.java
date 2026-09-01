package com.paraguacraft.pvp.modern.mixin;

import net.minecraft.client.util.Window;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

/**
 * Discord Overlay exige el título vanilla. 1.15+ añade
 * "Multijugador (servidor de terceros)" y Discord deja de detectarlo.
 */
@Mixin(Window.class)
public class MixinWindowTitle {

    private static final String WINDOW_TITLE = "Minecraft* 1.21.11";

    @ModifyVariable(method = "setTitle", at = @At("HEAD"), argsOnly = true)
    private String paraguacraft$discordDetectableTitle(String title) {
        if (title != null && title.equals(WINDOW_TITLE)) {
            return title;
        }
        return WINDOW_TITLE;
    }
}
