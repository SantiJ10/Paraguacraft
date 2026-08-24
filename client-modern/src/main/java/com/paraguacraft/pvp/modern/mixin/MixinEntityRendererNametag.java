package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.cosmetics.NametagOverlay;
import net.minecraft.client.render.command.OrderedRenderCommandQueue;
import net.minecraft.client.render.entity.EntityRenderer;
import net.minecraft.client.render.entity.state.EntityRenderState;
import net.minecraft.client.render.entity.state.PlayerEntityRenderState;
import net.minecraft.client.render.state.CameraRenderState;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.entity.Entity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Cancela el nametag vanilla de jugadores para redibujarlo con logo + vida.
 * El dibujo 3D vive en {@code NametagWorldRenderer} (Sodium/Lithium-safe).
 */
@Mixin(EntityRenderer.class)
public abstract class MixinEntityRendererNametag<T extends Entity, S extends EntityRenderState> {

    @Inject(method = "renderLabelIfPresent", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$replacePlayerLabel(
        S state,
        MatrixStack matrices,
        OrderedRenderCommandQueue queue,
        CameraRenderState cameraRenderState,
        CallbackInfo ci
    ) {
        if (!(state instanceof PlayerEntityRenderState)) {
            return;
        }
        if (NametagOverlay.isGuiEntityPass()) {
            ci.cancel();
            return;
        }
        if (NametagOverlay.shouldReplace3dLabel()) {
            ci.cancel();
        }
    }
}
