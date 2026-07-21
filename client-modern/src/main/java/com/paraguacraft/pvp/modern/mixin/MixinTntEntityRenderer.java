package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.render.command.OrderedRenderCommandQueue;
import net.minecraft.client.render.entity.TntEntityRenderer;
import net.minecraft.client.render.entity.state.TntEntityRenderState;
import net.minecraft.client.render.state.CameraRenderState;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.text.Text;
import net.minecraft.text.TextColor;
import net.minecraft.util.math.Vec3d;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Segundos restantes flotando sobre la TNT (paridad 1.8.9). */
@Mixin(TntEntityRenderer.class)
public class MixinTntEntityRenderer {

    @Inject(
        method = "render(Lnet/minecraft/client/render/entity/state/TntEntityRenderState;Lnet/minecraft/client/util/math/MatrixStack;Lnet/minecraft/client/render/command/OrderedRenderCommandQueue;Lnet/minecraft/client/render/state/CameraRenderState;)V",
        at = @At("RETURN")
    )
    private void paraguacraft$tntCountdown(
        TntEntityRenderState state,
        MatrixStack matrices,
        OrderedRenderCommandQueue queue,
        CameraRenderState cameraState,
        CallbackInfo ci
    ) {
        if (!ModernConfig.showTntCountdown || state.fuse <= 0.0F) {
            return;
        }
        float sec = state.fuse / 20.0F;
        Text label = Text.literal(String.format("%.1f", sec))
            .styled(s -> s.withColor(TextColor.fromRgb(0xFF5555)));
        Vec3d pos = new Vec3d(state.x, state.y + state.height + 0.45, state.z);
        queue.submitLabel(matrices, pos, 0, label, true, state.light, state.squaredDistanceToCamera, cameraState);
    }
}
