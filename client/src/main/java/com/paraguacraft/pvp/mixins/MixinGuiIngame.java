package com.paraguacraft.pvp.mixins;

import com.paraguacraft.pvp.core.ScoreboardFilter;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.GuiIngame;
import net.minecraft.client.gui.ScaledResolution;
import net.minecraft.scoreboard.ScoreObjective;
import net.minecraft.util.EnumChatFormatting;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Constant;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyConstant;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(GuiIngame.class)
public class MixinGuiIngame {

    @Inject(method = "renderScoreboard", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$cancelIfHidden(ScoreObjective objective, ScaledResolution sr, CallbackInfo ci) {
        if (!ModConfig.scoreboardEnabled) {
            ci.cancel();
        }
    }

    @ModifyConstant(method = "renderScoreboard", constant = @Constant(intValue = 1342177280))
    private int paraguacraft$transparentBg(int originalColor) {
        return ModConfig.scoreboardTransparentBg ? 0 : originalColor;
    }

    @Redirect(
        method = "renderScoreboard",
        at = @At(value = "INVOKE", target = "Lnet/minecraft/client/gui/FontRenderer;drawString(Ljava/lang/String;III)I")
    )
    private int paraguacraft$filterDraw(FontRenderer fr, String text, int x, int y, int color) {
        if (shouldSkip(text)) {
            return x + fr.getStringWidth(text);
        }
        return fr.drawString(text, x, y, color);
    }

    private static boolean shouldSkip(String text) {
        String plain = EnumChatFormatting.getTextWithoutFormattingCodes(text);
        if (ModConfig.scoreboardHideRedNumbers && ScoreboardFilter.isScoreColumnNumber(text)) {
            return true;
        }
        if (ModConfig.scoreboardHideStats && ScoreboardFilter.shouldHide(plain)) {
            return true;
        }
        return false;
    }
}
