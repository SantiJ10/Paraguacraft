package com.paraguacraft.pvp.modern.mixin.performance;

import com.paraguacraft.pvp.modern.core.CullHelper;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.command.OrderedRenderCommandQueue;
import net.minecraft.client.render.entity.PlayerEntityRenderer;
import net.minecraft.client.render.entity.state.PlayerEntityRenderState;
import net.minecraft.client.render.state.CameraRenderState;
import net.minecraft.client.util.math.MatrixStack;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Nametag cull + LOD en jugadores (usa distancia ya calculada en render state). */
@Mixin(PlayerEntityRenderer.class)
public abstract class MixinPlayerEntityRendererNametagCull {

    @Inject(method = "renderLabelIfPresent", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$cullNametags(
        PlayerEntityRenderState state,
        MatrixStack matrices,
        OrderedRenderCommandQueue queue,
        CameraRenderState cameraState,
        CallbackInfo ci
    ) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (CullHelper.shouldCullNametag(state, client.targetedEntity)) {
            ci.cancel();
        }
    }
}
