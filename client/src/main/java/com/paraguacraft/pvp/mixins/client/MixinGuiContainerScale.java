package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.ScaledResolution;
import net.minecraft.client.gui.inventory.GuiContainer;
import net.minecraft.client.renderer.GlStateManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(GuiContainer.class)
public class MixinGuiContainerScale {

    @Unique
    private boolean paraguacraft$scaled;

    @Inject(method = "drawScreen", at = @At("HEAD"))
    private void paraguacraft$beginScale(int mouseX, int mouseY, float partialTicks, CallbackInfo ci) {
        paraguacraft$scaled = false;
        float s = ModConfig.inventoryScaleFactor();
        if (s == 1.0F) {
            return;
        }
        paraguacraft$scaled = true;
        float[] c = paraguacraft$center();
        GlStateManager.pushMatrix();
        GlStateManager.translate(c[0], c[1], 0.0F);
        GlStateManager.scale(s, s, 1.0F);
        GlStateManager.translate(-c[0], -c[1], 0.0F);
    }

    @Inject(method = "drawScreen", at = @At("RETURN"))
    private void paraguacraft$endScale(int mouseX, int mouseY, float partialTicks, CallbackInfo ci) {
        if (paraguacraft$scaled) {
            GlStateManager.popMatrix();
            paraguacraft$scaled = false;
        }
    }

    @ModifyVariable(method = "drawScreen", at = @At("HEAD"), argsOnly = true, ordinal = 0)
    private int paraguacraft$remapDrawX(int mouseX) {
        return paraguacraft$unmap(mouseX, true);
    }

    @ModifyVariable(method = "drawScreen", at = @At("HEAD"), argsOnly = true, ordinal = 1)
    private int paraguacraft$remapDrawY(int mouseY) {
        return paraguacraft$unmap(mouseY, false);
    }

    @ModifyVariable(method = "mouseClicked", at = @At("HEAD"), argsOnly = true, ordinal = 0)
    private int paraguacraft$remapClickX(int mouseX) {
        return paraguacraft$unmap(mouseX, true);
    }

    @ModifyVariable(method = "mouseClicked", at = @At("HEAD"), argsOnly = true, ordinal = 1)
    private int paraguacraft$remapClickY(int mouseY) {
        return paraguacraft$unmap(mouseY, false);
    }

    @Unique
    private int paraguacraft$unmap(int value, boolean xAxis) {
        float s = ModConfig.inventoryScaleFactor();
        if (s == 1.0F) {
            return value;
        }
        float[] c = paraguacraft$center();
        float origin = xAxis ? c[0] : c[1];
        return Math.round((value - origin) / s + origin);
    }

    @Unique
    private static float[] paraguacraft$center() {
        ScaledResolution sr = new ScaledResolution(Minecraft.getMinecraft());
        return new float[] { sr.getScaledWidth() / 2.0F, sr.getScaledHeight() / 2.0F };
    }
}
