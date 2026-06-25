package com.paraguacraft.pvp.mixins.performance;

import com.paraguacraft.pvp.core.PerformanceConfig;
import net.minecraft.client.particle.EffectRenderer;
import net.minecraft.client.particle.EntityFX;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Limita partículas activas — evita caídas de FPS en combate / lluvia / encantamientos.
 * Usa contadores propios (sin {@code @Shadow fxLayers}) para compatibilidad con OptiFine.
 */
@Mixin(EffectRenderer.class)
public class MixinEffectRenderer {

    @Unique
    private static int paraguacraft$spawnedThisTick;

    @Unique
    private static int paraguacraft$estimatedActive;

    @Inject(method = "updateEffects", at = @At("HEAD"))
    private void paraguacraft$resetSpawnCounter(CallbackInfo ci) {
        paraguacraft$spawnedThisTick = 0;
        if (paraguacraft$estimatedActive > 0) {
            paraguacraft$estimatedActive = Math.max(0, paraguacraft$estimatedActive - paraguacraft$estimatedActive / 16);
        }
    }

    @Inject(method = "addEffect", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$limitParticles(EntityFX effect, CallbackInfo ci) {
        if (!PerformanceConfig.particleLimit) {
            return;
        }
        if (paraguacraft$estimatedActive >= PerformanceConfig.maxParticles) {
            ci.cancel();
            return;
        }
        if (paraguacraft$spawnedThisTick >= PerformanceConfig.maxParticlesPerTick) {
            ci.cancel();
            return;
        }
        paraguacraft$spawnedThisTick++;
        paraguacraft$estimatedActive++;
    }
}
