package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.hud.InGameHud;
import net.minecraft.client.render.RenderTickCounter;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(InGameHud.class)
public class MixinInGameHudCrosshair {

    @Inject(method = "renderCrosshair", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$customCrosshair(DrawContext context, RenderTickCounter tickCounter, CallbackInfo ci) {
        if (ModernConfig.crosshairMode == 0) {
            return;
        }
        ci.cancel();
        int cx = context.getScaledWindowWidth() / 2;
        int cy = context.getScaledWindowHeight() / 2;
        int color = UiTheme.accent();
        switch (ModernConfig.crosshairMode) {
            case 1 -> {
                context.fill(cx - 4, cy - 1, cx + 4, cy + 1, color);
                context.fill(cx - 1, cy - 4, cx + 1, cy + 4, color);
            }
            case 2 -> {
                int gap = 2;
                int size = 5;
                context.fill(cx - gap - size, cy - 1, cx - gap, cy + 1, color);
                context.fill(cx + gap, cy - 1, cx + gap + size, cy + 1, color);
                context.fill(cx - 1, cy - gap - size, cx + 1, cy - gap, color);
                context.fill(cx - 1, cy + gap, cx + 1, cy + gap + size, color);
            }
            case 3 -> context.fill(cx - 1, cy - 1, cx + 1, cy + 1, color);
            case 4 -> {
                context.fill(cx - 2, cy - 2, cx + 2, cy + 2, color);
                context.fill(cx - 1, cy - 3, cx + 1, cy + 3, color);
                context.fill(cx - 3, cy - 1, cx + 3, cy + 1, color);
            }
            default -> {}
        }
    }
}
