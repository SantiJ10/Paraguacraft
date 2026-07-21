package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.render.LightmapTextureManager;
import net.minecraft.world.dimension.DimensionType;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** Fullbright real en 1.21.11: gamma solo no alcanza; forzamos brillo maximo del lightmap. */
@Mixin(LightmapTextureManager.class)
public class MixinLightmapTextureManager {

    @Inject(method = "getBrightness(FI)F", at = @At("HEAD"), cancellable = true)
    private static void paraguacraft$fullbright(float ambientLight, int lightLevel, CallbackInfoReturnable<Float> cir) {
        if (ModernConfig.fullbright) {
            cir.setReturnValue(1.0F);
        }
    }

    @Inject(method = "getBrightness(Lnet/minecraft/world/dimension/DimensionType;I)F", at = @At("HEAD"), cancellable = true)
    private static void paraguacraft$fullbrightDim(DimensionType type, int lightLevel, CallbackInfoReturnable<Float> cir) {
        if (ModernConfig.fullbright) {
            cir.setReturnValue(1.0F);
        }
    }
}
