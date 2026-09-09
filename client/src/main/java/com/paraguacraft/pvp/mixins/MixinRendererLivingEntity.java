package com.paraguacraft.pvp.mixins;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.renderer.entity.RendererLivingEntity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;
import java.nio.FloatBuffer;

@Mixin(RendererLivingEntity.class)
public class MixinRendererLivingEntity {

    @Redirect(method = "setBrightness", at = @At(value = "INVOKE", target = "Ljava/nio/FloatBuffer;put(F)Ljava/nio/FloatBuffer;", ordinal = 0))
    private FloatBuffer customHitColorRed(FloatBuffer instance, float f) {
        return instance.put(ModConfig.hitColorEnabled ? ModConfig.hitColorR() : f);
    }

    @Redirect(method = "setBrightness", at = @At(value = "INVOKE", target = "Ljava/nio/FloatBuffer;put(F)Ljava/nio/FloatBuffer;", ordinal = 1))
    private FloatBuffer customHitColorGreen(FloatBuffer instance, float f) {
        return instance.put(ModConfig.hitColorEnabled ? ModConfig.hitColorG() : f);
    }

    @Redirect(method = "setBrightness", at = @At(value = "INVOKE", target = "Ljava/nio/FloatBuffer;put(F)Ljava/nio/FloatBuffer;", ordinal = 2))
    private FloatBuffer customHitColorBlue(FloatBuffer instance, float f) {
        return instance.put(ModConfig.hitColorEnabled ? ModConfig.hitColorB() : f);
    }
}
