package com.paraguacraft.pvp.mixins.performance;

import com.paraguacraft.pvp.core.PerformanceConfig;
import net.minecraft.client.renderer.culling.ICamera;
import net.minecraft.client.renderer.entity.RenderManager;
import net.minecraft.entity.Entity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Culling adicional de entidades: evita renderizar entidades muy lejanas
 * que aún pasaron el chequeo vanilla (útil en hubs con muchos jugadores).
 */
@Mixin(RenderManager.class)
public class MixinRenderManager {

    @Inject(
        method = "shouldRender",
        at = @At("HEAD"),
        cancellable = true
    )
    private void paraguacraft$cullDistantEntities(
        Entity entity,
        ICamera camera,
        double camX,
        double camY,
        double camZ,
        CallbackInfoReturnable<Boolean> cir
    ) {
        if (!PerformanceConfig.entityCull || entity == null) {
            return;
        }
        // Nunca cullar al jugador local ni entidades montadas sobre él.
        net.minecraft.client.Minecraft mc = net.minecraft.client.Minecraft.getMinecraft();
        if (entity == mc.thePlayer || entity.ridingEntity == mc.thePlayer) {
            return;
        }
        double dx = entity.posX - camX;
        double dy = entity.posY - camY;
        double dz = entity.posZ - camZ;
        double distSq = dx * dx + dy * dy + dz * dz;
        if (distSq > PerformanceConfig.entityCullDistanceSq && !camera.isBoundingBoxInFrustum(entity.getEntityBoundingBox())) {
            cir.setReturnValue(false);
        }
    }
}
