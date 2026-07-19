package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.WindowedFullscreenManager;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.util.Window;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Window.class)
public class MixinWindowFullscreen {

    @Inject(method = "toggleFullscreen", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$windowedFullscreen(CallbackInfo ci) {
        if (!ModernConfig.windowedFullscreen) {
            return;
        }
        WindowedFullscreenManager.toggle(MinecraftClient.getInstance());
        ci.cancel();
    }
}
