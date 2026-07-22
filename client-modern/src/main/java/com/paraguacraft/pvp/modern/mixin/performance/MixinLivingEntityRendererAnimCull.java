package com.paraguacraft.pvp.modern.mixin.performance;

import com.paraguacraft.pvp.modern.core.CullHelper;
import net.minecraft.client.render.entity.LivingEntityRenderer;
import net.minecraft.client.render.entity.state.LivingEntityRenderState;
import net.minecraft.entity.LivingEntity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Congela animaciones de entidades lejanas (limb swing). */
@Mixin(LivingEntityRenderer.class)
public abstract class MixinLivingEntityRendererAnimCull {

    @Inject(method = "updateRenderState", at = @At("RETURN"))
    private void paraguacraft$freezeDistantAnimations(
        LivingEntity entity,
        LivingEntityRenderState state,
        float tickProgress,
        CallbackInfo ci
    ) {
        if (state == null || !CullHelper.shouldFreezeEntityAnim(entity)) {
            return;
        }
        state.limbSwingAnimationProgress = 0.0F;
        state.limbSwingAmplitude = 0.0F;
    }
}
