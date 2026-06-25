package com.paraguacraft.pvp.mixins.performance;

import com.paraguacraft.pvp.core.PerformanceConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.entity.Render;
import net.minecraft.entity.Entity;
import net.minecraft.entity.player.EntityPlayer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * No dibuja nametags de jugadores lejanos (estilo Lunar entity/nametag culling).
 */
@Mixin(Render.class)
public class MixinNametagCull {

    @Inject(
        method = "renderLivingLabel(Lnet/minecraft/entity/Entity;Ljava/lang/String;DDDI)V",
        at = @At("HEAD"),
        cancellable = true,
        require = 0
    )
    private void paraguacraft$cullDistantNametags(
        Entity entity,
        String label,
        double x,
        double y,
        double z,
        int maxDistance,
        CallbackInfo ci
    ) {
        if (!PerformanceConfig.nametagCull || !(entity instanceof EntityPlayer)) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null || entity == mc.thePlayer) {
            return;
        }
        double distSq = mc.thePlayer.getDistanceSqToEntity(entity);
        if (distSq > PerformanceConfig.nametagCullDistanceSq) {
            ci.cancel();
            return;
        }
        if (PerformanceConfig.nametagLod && distSq > PerformanceConfig.nametagLodDistanceSq) {
            if (mc.objectMouseOver == null || mc.objectMouseOver.entityHit != entity) {
                ci.cancel();
            }
        }
    }
}
