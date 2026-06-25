package com.paraguacraft.pvp.mixins.animations;

import com.paraguacraft.pvp.core.PerformanceConfig;
import net.minecraft.client.entity.EntityPlayerSP;
import net.minecraft.entity.EntityLivingBase;
import net.minecraft.util.MathHelper;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Swing más rápido estilo 1.7 — solo aplica al jugador local.
 */
@Mixin(EntityLivingBase.class)
public abstract class MixinEntityLivingBaseSwing {

    @Shadow public int swingProgressInt;

    @Inject(method = "getSwingProgress", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$fasterSwingProgress(float partialTicks, CallbackInfoReturnable<Float> cir) {
        if (!PerformanceConfig.oldAnimations) {
            return;
        }
        if (!((Object) this instanceof EntityPlayerSP)) {
            return;
        }
        if (this.swingProgressInt <= 0) {
            return;
        }
        float progress = (float) this.swingProgressInt + partialTicks;
        float max = 5.0F;
        float value = progress / max;
        if (value >= 1.0F) {
            cir.setReturnValue(1.0F);
            return;
        }
        float eased = MathHelper.sin(value * (float) Math.PI * 0.5F);
        cir.setReturnValue(eased);
    }
}
