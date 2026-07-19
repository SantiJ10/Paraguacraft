package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.gui.CustomPauseScreen;
import com.paraguacraft.pvp.modern.gui.CustomTitleScreen;
import net.minecraft.client.gui.screen.GameMenuScreen;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.TitleScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(net.minecraft.client.MinecraftClient.class)
public class MixinMinecraftClient {

    @ModifyVariable(method = "setScreen", at = @At("HEAD"), argsOnly = true)
    private Screen paraguacraft$replaceMenu(Screen screen) {
        if (screen == null) {
            return null;
        }
        if (screen instanceof TitleScreen) {
            return new CustomTitleScreen();
        }
        if (screen instanceof GameMenuScreen && !(screen instanceof CustomPauseScreen)) {
            return new CustomPauseScreen(true);
        }
        return screen;
    }

    @Inject(method = "tick", at = @At("HEAD"))
    private void paraguacraft$healBorderless(CallbackInfo ci) {
        if (com.paraguacraft.pvp.modern.config.ModernConfig.windowedFullscreen
            && com.paraguacraft.pvp.modern.core.WindowedFullscreenManager.isActive()) {
            com.paraguacraft.pvp.modern.core.WindowedFullscreenManager.heal((net.minecraft.client.MinecraftClient) (Object) this);
        }
    }
}
