package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.ingame.HandledScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(HandledScreen.class)
public class MixinHandledScreenScale {

    @Unique
    private boolean paraguacraft$scaled;

    @Inject(method = "render", at = @At("HEAD"), require = 0)
    private void paraguacraft$beginScale(DrawContext context, int mouseX, int mouseY, float delta, CallbackInfo ci) {
        paraguacraft$scaled = false;
        float s = ModernConfig.inventoryScaleFactor();
        if (s == 1.0F) {
            return;
        }
        paraguacraft$scaled = true;
        float w = context.getScaledWindowWidth() / 2.0F;
        float h = context.getScaledWindowHeight() / 2.0F;
        context.getMatrices().pushMatrix();
        context.getMatrices().translate(w, h);
        context.getMatrices().scale(s, s);
        context.getMatrices().translate(-w, -h);
    }

    @Inject(method = "render", at = @At("RETURN"), require = 0)
    private void paraguacraft$endScale(DrawContext context, int mouseX, int mouseY, float delta, CallbackInfo ci) {
        if (paraguacraft$scaled) {
            context.getMatrices().popMatrix();
            paraguacraft$scaled = false;
        }
    }

    @ModifyVariable(method = "render", at = @At("HEAD"), argsOnly = true, ordinal = 0, require = 0)
    private int paraguacraft$remapX(int mouseX) {
        return paraguacraft$unmap(mouseX, true);
    }

    @ModifyVariable(method = "render", at = @At("HEAD"), argsOnly = true, ordinal = 1, require = 0)
    private int paraguacraft$remapY(int mouseY) {
        return paraguacraft$unmap(mouseY, false);
    }

    @Unique
    private int paraguacraft$unmap(int value, boolean xAxis) {
        float s = ModernConfig.inventoryScaleFactor();
        if (s == 1.0F) {
            return value;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        float origin = xAxis
            ? client.getWindow().getScaledWidth() / 2.0F
            : client.getWindow().getScaledHeight() / 2.0F;
        return Math.round((value - origin) / s + origin);
    }
}
