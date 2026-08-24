package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.cosmetics.NametagOverlay;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.ingame.HandledScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(HandledScreen.class)
public abstract class MixinHandledScreenWatermark {

    @Inject(method = "render", at = @At("TAIL"))
    private void paraguacraft$watermark(DrawContext context, int mouseX, int mouseY, float delta, CallbackInfo ci) {
        HandledScreen<?> self = (HandledScreen<?>) (Object) this;
        if (!NametagOverlay.isWatermarkScreen(self)) {
            return;
        }
        NametagOverlay.drawWatermark(context, self.width, self.height);
    }
}
