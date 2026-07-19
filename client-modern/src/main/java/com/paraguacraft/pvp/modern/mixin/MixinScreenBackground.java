package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.gui.MenuBackground;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Fondo Paraguacraft en menus vanilla (multijugador, opciones, conexion). */
@Mixin(Screen.class)
public class MixinScreenBackground {

    @Inject(method = "renderBackground", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$menuBackground(DrawContext context, int mouseX, int mouseY, float delta, CallbackInfo ci) {
        Screen self = (Screen) (Object) this;
        if (!MenuBackground.shouldReplace(self)) {
            return;
        }
        MenuBackground.draw(self, context, mouseX, mouseY, delta, false);
        ci.cancel();
    }
}
