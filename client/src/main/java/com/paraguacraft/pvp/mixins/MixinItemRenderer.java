package com.paraguacraft.pvp.mixins;

import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.ItemRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ItemRenderer.class)
public class MixinItemRenderer {
    
    @Inject(method = "renderFireInFirstPerson", at = @At("HEAD"))
    private void onRenderFire(CallbackInfo ci) {
        // Empuja el fuego hacia abajo en el eje Y para limpiar tu campo de visión
        GlStateManager.translate(0.0F, -0.35F, 0.0F);
    }
}