package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.GameRenderer;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(GameRenderer.class)
public class MixinGameRendererFov {

    @Shadow @Final private MinecraftClient client;

    @Inject(method = "getFov", at = @At("RETURN"), cancellable = true)
    private void paraguacraft$staticFov(CallbackInfoReturnable<Float> cir) {
        if (!ModernConfig.dynamicFov) {
            cir.setReturnValue((float) client.options.getFov().getValue());
        }
    }
}
