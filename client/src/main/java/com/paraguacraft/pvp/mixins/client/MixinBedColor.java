package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.modules.BedColorHelper;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.block.state.IBlockState;
import net.minecraft.client.renderer.BlockRendererDispatcher;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.WorldRenderer;
import net.minecraft.util.BlockPos;
import net.minecraft.world.IBlockAccess;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(BlockRendererDispatcher.class)
public class MixinBedColor {

    @Inject(
        method = "renderBlock(Lnet/minecraft/block/state/IBlockState;Lnet/minecraft/util/BlockPos;Lnet/minecraft/world/IBlockAccess;Lnet/minecraft/client/renderer/WorldRenderer;)Z",
        at = @At("HEAD")
    )
    private void paraguacraft$tintBed(
        IBlockState state,
        BlockPos pos,
        IBlockAccess blockAccess,
        WorldRenderer worldRenderer,
        CallbackInfoReturnable<Boolean> cir
    ) {
        if (!ModConfig.coloredBeds || !BedColorHelper.isBedBlock(state)) {
            return;
        }
        float[] rgb = BedColorHelper.getColor(blockAccess, pos);
        GlStateManager.color(rgb[0], rgb[1], rgb[2], 1.0F);
    }

    @Inject(
        method = "renderBlock(Lnet/minecraft/block/state/IBlockState;Lnet/minecraft/util/BlockPos;Lnet/minecraft/world/IBlockAccess;Lnet/minecraft/client/renderer/WorldRenderer;)Z",
        at = @At("RETURN")
    )
    private void paraguacraft$resetBedTint(CallbackInfoReturnable<Boolean> cir) {
        if (ModConfig.coloredBeds) {
            GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
        }
    }
}
