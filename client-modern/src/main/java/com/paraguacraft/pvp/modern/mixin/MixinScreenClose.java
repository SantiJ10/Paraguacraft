package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.gui.CustomTitleScreen;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.TitleScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Tras cerrar una pantalla, reemplaza TitleScreen vanilla por el menu Paraguacraft. */
@Mixin(Screen.class)
public class MixinScreenClose {

    @Inject(method = "close", at = @At("TAIL"))
    private void paraguacraft$redirectAfterClose(CallbackInfo ci) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.currentScreen instanceof TitleScreen) {
            client.setScreen(new CustomTitleScreen());
        }
    }
}
