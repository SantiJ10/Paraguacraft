package com.paraguacraft.pvp.mixins.client;

import net.minecraft.client.gui.GuiSlot;
import net.minecraft.client.renderer.GlStateManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

/**
 * Cambia el marco blanco de selección de listas (servers, idiomas, etc.) a celeste.
 */
@Mixin(GuiSlot.class)
public abstract class MixinGuiSlotSelection {

    @Redirect(
        method = "drawSelectionBox",
        at = @At(
            value = "INVOKE",
            target = "Lnet/minecraft/client/renderer/GlStateManager;color(FFFF)V"
        )
    )
    private void paraguacraft$celesteSelection(float r, float g, float b, float a) {
        // El marco de selección en vanilla es blanco (1,1,1); pasarlo a cian.
        if (r > 0.95f && g > 0.95f && b > 0.95f) {
            GlStateManager.color(0f, 0.9f, 1f, a);
        } else {
            GlStateManager.color(r, g, b, a);
        }
    }
}
