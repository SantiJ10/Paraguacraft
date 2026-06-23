package com.paraguacraft.pvp.mixins;

import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.GuiIngame;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Constant;
import org.spongepowered.asm.mixin.injection.ModifyConstant;
import org.spongepowered.asm.mixin.injection.Redirect;
import com.paraguacraft.pvp.modules.ModConfig; // Importamos la config

@Mixin(GuiIngame.class)
public class MixinGuiIngame {
    
    @Redirect(method = "renderScoreboard", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/gui/FontRenderer;drawString(Ljava/lang/String;III)I"))
    private int hideRedNumbersCompletely(FontRenderer fontRenderer, String text, int x, int y, int color) {
        // Si el usuario quiere el fondo transparente, sacamos los numeros. Si no, los dejamos.
        if (ModConfig.transparentScoreboard && color == 553648127) {
            return x + fontRenderer.getStringWidth(text); 
        }
        return fontRenderer.drawString(text, x, y, color);
    }

    @ModifyConstant(method = "renderScoreboard", constant = @Constant(intValue = 1342177280))
    private int toggleScoreboardBackground(int originalColor) {
        // Si está activado, devolvemos 0 (invisible). Si está desactivado, devolvemos el original.
        return ModConfig.transparentScoreboard ? 0 : originalColor; 
    }
}