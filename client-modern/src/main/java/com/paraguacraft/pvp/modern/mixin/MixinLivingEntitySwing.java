package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.animations.OldAnimations;
import net.minecraft.client.network.ClientPlayerEntity;
import net.minecraft.entity.LivingEntity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** Swing mas rapido estilo 1.7 (solo jugador local). */
@Mixin(LivingEntity.class)
public abstract class MixinLivingEntitySwing {

    @Shadow private int handSwingTicks;

    @Inject(method = "getHandSwingProgress", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$fasterSwing(float tickProgress, CallbackInfoReturnable<Float> cir) {
        if (!OldAnimations.enabled()) {
            return;
        }
        if (!((Object) this instanceof ClientPlayerEntity)) {
            return;
        }
        if (handSwingTicks <= 0) {
            return;
        }
        float progress = handSwingTicks + tickProgress;
        float max = 5.0F;
        float value = progress / max;
        if (value >= 1.0F) {
            cir.setReturnValue(1.0F);
            return;
        }
        cir.setReturnValue((float) Math.sin(value * Math.PI * 0.5));
    }
}
