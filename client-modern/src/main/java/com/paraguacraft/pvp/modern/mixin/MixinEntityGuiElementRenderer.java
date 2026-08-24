package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.cosmetics.NametagOverlay;
import net.minecraft.client.gui.render.EntityGuiElementRenderer;
import net.minecraft.client.gui.render.state.special.EntityGuiElementRenderState;
import net.minecraft.client.util.math.MatrixStack;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * El preview 3D del inventario se encola con {@code DrawContext.addEntity} y se
 * dibuja acá. Envolver este render cancela el nametag 3D sobre el modelo GUI
 * (el “fantasma” de 1.8.9).
 */
@Mixin(EntityGuiElementRenderer.class)
public abstract class MixinEntityGuiElementRenderer {

    @Inject(method = "render", at = @At("HEAD"))
    private void paraguacraft$beginGuiEntity(EntityGuiElementRenderState state, MatrixStack matrices, CallbackInfo ci) {
        NametagOverlay.beginGuiEntityPass();
    }

    @Inject(method = "render", at = @At("RETURN"))
    private void paraguacraft$endGuiEntity(EntityGuiElementRenderState state, MatrixStack matrices, CallbackInfo ci) {
        NametagOverlay.endGuiEntityPass();
    }
}
