package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.core.PerformanceConfig;
import net.minecraft.client.particle.Particle;
import net.minecraft.client.particle.ParticleManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ParticleManager.class)
public class MixinParticleManager {

    @Inject(method = "addParticle(Lnet/minecraft/client/particle/Particle;)V", at = @At("HEAD"), cancellable = true, require = 0)
    private void paraguacraft$filterParticles(Particle particle, CallbackInfo ci) {
        if (particle == null) {
            return;
        }
        String name = particle.getClass().getSimpleName();
        if (PerformanceConfig.hideExplosionParticles
            && (name.contains("Explosion") || name.contains("FireSmoke") || name.contains("Firework"))) {
            ci.cancel();
            return;
        }
        if (PerformanceConfig.hidePotionParticles
            && (name.contains("Spell") || name.contains("Effect") || name.contains("EntityEffect"))) {
            ci.cancel();
        }
    }
}
