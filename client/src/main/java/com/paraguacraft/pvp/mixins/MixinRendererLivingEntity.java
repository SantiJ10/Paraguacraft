package com.paraguacraft.pvp.mixins;

import net.minecraft.client.renderer.entity.RendererLivingEntity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;
import java.nio.FloatBuffer;

@Mixin(RendererLivingEntity.class)
public class MixinRendererLivingEntity {

    // Interceptamos la primera inyeccion de color (Canal Rojo) y la ponemos en 0
    @Redirect(method = "setBrightness", at = @At(value = "INVOKE", target = "Ljava/nio/FloatBuffer;put(F)Ljava/nio/FloatBuffer;", ordinal = 0))
    private FloatBuffer customHitColorRed(FloatBuffer instance, float f) {
        return instance.put(0.0F); 
    }

    // Interceptamos la segunda inyeccion (Canal Verde) y lo ponemos alto
    @Redirect(method = "setBrightness", at = @At(value = "INVOKE", target = "Ljava/nio/FloatBuffer;put(F)Ljava/nio/FloatBuffer;", ordinal = 1))
    private FloatBuffer customHitColorGreen(FloatBuffer instance, float f) {
        return instance.put(0.898F); 
    }

    // Interceptamos la tercera inyeccion (Canal Azul) y lo ponemos al maximo
    @Redirect(method = "setBrightness", at = @At(value = "INVOKE", target = "Ljava/nio/FloatBuffer;put(F)Ljava/nio/FloatBuffer;", ordinal = 2))
    private FloatBuffer customHitColorBlue(FloatBuffer instance, float f) {
        return instance.put(1.0F); 
    }
}