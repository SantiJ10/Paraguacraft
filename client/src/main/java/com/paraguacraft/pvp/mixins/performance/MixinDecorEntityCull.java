package com.paraguacraft.pvp.mixins.performance;

import com.paraguacraft.pvp.core.PerformanceConfig;
import net.minecraft.client.renderer.culling.ICamera;
import net.minecraft.client.renderer.entity.RenderManager;
import net.minecraft.entity.Entity;
import net.minecraft.entity.item.EntityArmorStand;
import net.minecraft.entity.item.EntityItemFrame;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(RenderManager.class)
public class MixinDecorEntityCull {

    @Inject(method = "shouldRender", at = @At("HEAD"), cancellable = true, require = 0)
    private void paraguacraft$cullDecorEntities(
        Entity entity,
        ICamera camera,
        double camX,
        double camY,
        double camZ,
        CallbackInfoReturnable<Boolean> cir
    ) {
        if (entity == null) {
            return;
        }
        boolean cullArmor = PerformanceConfig.armorStandCull && entity instanceof EntityArmorStand;
        boolean cullFrame = PerformanceConfig.itemFrameCull && entity instanceof EntityItemFrame;
        if (!cullArmor && !cullFrame) {
            return;
        }
        int distSqLimit = cullArmor
            ? PerformanceConfig.armorStandCullDistanceSq
            : PerformanceConfig.itemFrameCullDistanceSq;
        double dx = entity.posX - camX;
        double dy = entity.posY - camY;
        double dz = entity.posZ - camZ;
        if (dx * dx + dy * dy + dz * dz > distSqLimit) {
            cir.setReturnValue(false);
        }
    }
}
