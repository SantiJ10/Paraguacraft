package com.paraguacraft.pvp.modern.mixin;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.ingame.HandledScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Fondo negro ~60% detras del inventario / containers (estilo Dewiers/PvP). */
@Mixin(HandledScreen.class)
public class MixinHandledScreenDim {

    @Inject(method = "renderBackground", at = @At("TAIL"))
    private void paraguacraft$dimInventory(DrawContext context, int mouseX, int mouseY, float delta, CallbackInfo ci) {
        HandledScreen<?> self = (HandledScreen<?>) (Object) this;
        // 0x99000000 ~= 60% negro
        context.fill(0, 0, self.width, self.height, 0x99000000);
    }
}
