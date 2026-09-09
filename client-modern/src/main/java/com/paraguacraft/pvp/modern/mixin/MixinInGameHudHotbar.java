package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.hud.InGameHud;
import net.minecraft.client.render.RenderTickCounter;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(InGameHud.class)
public class MixinInGameHudHotbar {

    @Unique
    private boolean paraguacraft$hotbarScaled;

    @Inject(method = "renderHotbar", at = @At("HEAD"), require = 0)
    private void paraguacraft$beginHotbar(DrawContext context, RenderTickCounter tickCounter, CallbackInfo ci) {
        float s = ModernConfig.hotbarScaleFactor();
        paraguacraft$hotbarScaled = s != 1.0F;
        if (paraguacraft$hotbarScaled) {
            float w = context.getScaledWindowWidth();
            float h = context.getScaledWindowHeight();
            context.getMatrices().pushMatrix();
            context.getMatrices().translate(w / 2.0F, h);
            context.getMatrices().scale(s, s);
            context.getMatrices().translate(-w / 2.0F, -h);
        }
    }

    @Inject(method = "renderHotbar", at = @At("RETURN"), require = 0)
    private void paraguacraft$endHotbar(DrawContext context, RenderTickCounter tickCounter, CallbackInfo ci) {
        if (paraguacraft$hotbarScaled) {
            context.getMatrices().popMatrix();
            paraguacraft$hotbarScaled = false;
        }
    }
}
