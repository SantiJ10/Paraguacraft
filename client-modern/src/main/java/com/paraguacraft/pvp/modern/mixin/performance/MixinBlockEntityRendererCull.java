package com.paraguacraft.pvp.modern.mixin.performance;

import com.paraguacraft.pvp.modern.core.CullHelper;
import net.minecraft.block.entity.BlockEntity;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.block.entity.BlockEntityRenderer;
import net.minecraft.util.math.Vec3d;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** No renderiza block entities lejanas (cofres, carteles, etc.). */
@Mixin(BlockEntityRenderer.class)
public interface MixinBlockEntityRendererCull {

    @Inject(method = "isInRenderDistance", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$cullBlockEntities(
        BlockEntity blockEntity,
        Vec3d pos,
        CallbackInfoReturnable<Boolean> cir
    ) {
        if (CullHelper.shouldCullBlockEntity(blockEntity == null ? null : blockEntity.getPos(), MinecraftClient.getInstance())) {
            cir.setReturnValue(false);
        }
    }
}
