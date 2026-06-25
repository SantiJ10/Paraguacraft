package com.paraguacraft.pvp.mixins;

import com.paraguacraft.pvp.animations.OldAnimations;
import com.paraguacraft.pvp.core.PerformanceConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.ItemRenderer;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Módulo 1: fuego en 1ª persona + animaciones 1.7 (blockhit / comer).
 * El swing se acelera vía {@link com.paraguacraft.pvp.mixins.animations.MixinEntityLivingBaseSwing}.
 */
@Mixin(ItemRenderer.class)
public abstract class MixinItemRenderer {

    @Shadow @Final private Minecraft mc;

    @Inject(method = "renderFireInFirstPerson", at = @At("HEAD"))
    private void paraguacraft$lowerFireOverlay(CallbackInfo ci) {
        GlStateManager.translate(0.0F, -0.35F, 0.0F);
    }

    @Inject(method = "renderItemInFirstPerson", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$oldUseAnimations(float partialTicks, CallbackInfo ci) {
        if (!PerformanceConfig.oldAnimations) {
            return;
        }
        ItemRenderer self = (ItemRenderer) (Object) this;
        if (OldAnimations.renderFirstPerson(self, this.mc, partialTicks)) {
            ci.cancel();
        }
    }
}
