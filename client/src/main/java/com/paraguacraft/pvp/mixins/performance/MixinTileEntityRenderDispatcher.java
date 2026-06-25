package com.paraguacraft.pvp.mixins.performance;

import com.paraguacraft.pvp.core.PerformanceConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.tileentity.TileEntityRendererDispatcher;
import net.minecraft.tileentity.TileEntity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * No renderiza tile entities lejanas (marcos, cofres, etc.).
 */
@Mixin(TileEntityRendererDispatcher.class)
public class MixinTileEntityRenderDispatcher {

    @Inject(
        method = "renderTileEntityAt(Lnet/minecraft/tileentity/TileEntity;DDDF)V",
        at = @At("HEAD"),
        cancellable = true,
        require = 0
    )
    private void paraguacraft$cullDistantTileEntities(
        TileEntity tileEntity,
        double x,
        double y,
        double z,
        float partialTicks,
        CallbackInfo ci
    ) {
        if (!shouldCull(tileEntity)) {
            return;
        }
        ci.cancel();
    }

    @Inject(
        method = "renderTileEntityAt(Lnet/minecraft/tileentity/TileEntity;DDDFI)V",
        at = @At("HEAD"),
        cancellable = true,
        require = 0
    )
    private void paraguacraft$cullDistantTileEntitiesStage(
        TileEntity tileEntity,
        double x,
        double y,
        double z,
        float partialTicks,
        int destroyStage,
        CallbackInfo ci
    ) {
        if (!shouldCull(tileEntity)) {
            return;
        }
        ci.cancel();
    }

    private static boolean shouldCull(TileEntity tileEntity) {
        if (!PerformanceConfig.blockEntityCull || tileEntity == null) {
            return false;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null) {
            return false;
        }
        double dx = tileEntity.getPos().getX() + 0.5D - mc.thePlayer.posX;
        double dy = tileEntity.getPos().getY() + 0.5D - mc.thePlayer.posY;
        double dz = tileEntity.getPos().getZ() + 0.5D - mc.thePlayer.posZ;
        return dx * dx + dy * dy + dz * dz > PerformanceConfig.blockEntityCullDistanceSq;
    }
}
