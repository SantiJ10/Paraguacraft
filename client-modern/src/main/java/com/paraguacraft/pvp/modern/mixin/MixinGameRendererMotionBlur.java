package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.core.MotionBlurRenderer;
import net.minecraft.client.render.GameRenderer;
import net.minecraft.client.render.RenderTickCounter;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Accumulation blur al terminar el mundo, antes del HUD. */
@Mixin(GameRenderer.class)
public class MixinGameRendererMotionBlur {

    @Inject(method = "renderWorld", at = @At("RETURN"), require = 0)
    private void paraguacraft$motionBlur(RenderTickCounter tickCounter, CallbackInfo ci) {
        MotionBlurRenderer.afterWorld();
    }
}
