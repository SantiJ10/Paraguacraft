package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.hud.InGameHud;
import net.minecraft.util.Identifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(InGameHud.class)
public class MixinInGameHudLowFire {

    private static final ThreadLocal<Float> FIRE_SHIFT = new ThreadLocal<>();

    @Inject(method = "renderOverlay", at = @At("HEAD"))
    private void paraguacraft$lowFireHead(DrawContext context, Identifier texture, float opacity, CallbackInfo ci) {
        if (!ModernConfig.lowFire || texture == null || !texture.getPath().contains("fire")) {
            FIRE_SHIFT.remove();
            return;
        }
        float shift = context.getScaledWindowHeight() * 0.18F;
        FIRE_SHIFT.set(shift);
        context.getMatrices().translate(0, shift);
    }

    @Inject(method = "renderOverlay", at = @At("RETURN"))
    private void paraguacraft$lowFireReturn(DrawContext context, Identifier texture, float opacity, CallbackInfo ci) {
        Float shift = FIRE_SHIFT.get();
        if (shift != null) {
            context.getMatrices().translate(0, -shift);
            FIRE_SHIFT.remove();
        }
    }
}
