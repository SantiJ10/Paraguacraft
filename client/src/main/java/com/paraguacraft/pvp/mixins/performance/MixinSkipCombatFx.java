package com.paraguacraft.pvp.mixins.performance;

import com.paraguacraft.pvp.core.PerformanceConfig;
import net.minecraft.client.renderer.EntityRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(EntityRenderer.class)
public class MixinSkipCombatFx {

    @Inject(method = "hurtCameraEffect", at = @At("HEAD"), cancellable = true, require = 0)
    private void paraguacraft$skipHurtCamera(float partialTicks, CallbackInfo ci) {
        if (PerformanceConfig.skipCombatFx) {
            ci.cancel();
        }
    }
}
