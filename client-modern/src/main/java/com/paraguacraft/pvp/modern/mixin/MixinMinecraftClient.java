package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.gui.CustomPauseScreen;
import com.paraguacraft.pvp.modern.gui.CustomTitleScreen;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.GameMenuScreen;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.TitleScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(MinecraftClient.class)
public class MixinMinecraftClient {

    @Inject(method = "setScreen", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$replaceMenu(Screen screen, CallbackInfo ci) {
        if (screen == null) {
            return;
        }
        MinecraftClient client = (MinecraftClient) (Object) this;
        if (screen.getClass() == TitleScreen.class && !(screen instanceof CustomTitleScreen)) {
            client.setScreen(new CustomTitleScreen());
            ci.cancel();
        } else if (screen.getClass() == GameMenuScreen.class && !(screen instanceof CustomPauseScreen)) {
            client.setScreen(new CustomPauseScreen(true));
            ci.cancel();
        }
    }
}
