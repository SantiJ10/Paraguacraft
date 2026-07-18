package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.render.entity.ItemEntityRenderer;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.entity.ItemEntity;
import net.minecraft.util.math.RotationAxis;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ItemEntityRenderer.class)
public class MixinItemEntityRenderer {

    @Inject(
        method = "render(Lnet/minecraft/client/render/entity/state/ItemEntityRenderState;Lnet/minecraft/client/util/math/MatrixStack;Lnet/minecraft/client/render/command/OrderedRenderCommandQueue;Lnet/minecraft/client/render/state/CameraRenderState;)V",
        at = @At("HEAD")
    )
    private void paraguacraft$itemBob(
        net.minecraft.client.render.entity.state.ItemEntityRenderState state,
        MatrixStack matrices,
        CallbackInfo ci
    ) {
        if (!ModernConfig.itemPhysics) {
            return;
        }
        float spin = (state.uniqueOffset + state.age) * 4.0F;
        matrices.multiply(RotationAxis.POSITIVE_Y.rotationDegrees(spin));
        matrices.translate(0.0F, (float) Math.sin((state.age + state.uniqueOffset) * 0.15F) * 0.08F, 0.0F);
    }
}
