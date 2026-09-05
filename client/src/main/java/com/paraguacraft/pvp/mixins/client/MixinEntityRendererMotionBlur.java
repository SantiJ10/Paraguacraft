package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.core.MotionBlurRenderer;
import net.minecraft.client.renderer.EntityRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Accumulation blur al terminar el mundo, antes del HUD. */
@Mixin(EntityRenderer.class)
public class MixinEntityRendererMotionBlur {

    @Inject(method = "renderWorld", at = @At("RETURN"))
    private void paraguacraft$motionBlur(float partialTicks, long finishTimeNano, CallbackInfo ci) {
        MotionBlurRenderer.afterWorld();
    }
}
