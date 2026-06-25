package com.paraguacraft.pvp.mixins.performance;

import com.paraguacraft.pvp.core.PerformanceConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.entity.RendererLivingEntity;
import net.minecraft.entity.EntityLivingBase;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

/**
 * Congela animaciones de entidades lejanas (OptiFine-safe, sin @ModifyArgs).
 */
@Mixin(RendererLivingEntity.class)
public class MixinEntityAnimCull {

    @ModifyVariable(
        method = "renderModel(Lnet/minecraft/entity/EntityLivingBase;FFFFFF)V",
        at = @At("HEAD"),
        argsOnly = true,
        ordinal = 1,
        require = 0
    )
    private float paraguacraft$freezeLimbSwing(float limbSwing, EntityLivingBase entity) {
        return shouldFreeze(entity) ? 0.0F : limbSwing;
    }

    @ModifyVariable(
        method = "renderModel(Lnet/minecraft/entity/EntityLivingBase;FFFFFF)V",
        at = @At("HEAD"),
        argsOnly = true,
        ordinal = 2,
        require = 0
    )
    private float paraguacraft$freezeLimbSwingAmount(float limbSwingAmount, EntityLivingBase entity) {
        return shouldFreeze(entity) ? 0.0F : limbSwingAmount;
    }

    private static boolean shouldFreeze(EntityLivingBase entity) {
        if (!PerformanceConfig.entityAnimCull || entity == null) {
            return false;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null || entity == mc.thePlayer) {
            return false;
        }
        return mc.thePlayer.getDistanceSqToEntity(entity) > PerformanceConfig.entityAnimCullDistanceSq;
    }
}
