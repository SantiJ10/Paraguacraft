package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.NickFinderManager;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.hud.PlayerListHud;
import net.minecraft.text.Text;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyArg;

@Mixin(PlayerListHud.class)
public class MixinPlayerListHudNickTeam {

    @ModifyArg(
        method = "render",
        at = @At(
            value = "INVOKE",
            target = "Lnet/minecraft/client/gui/DrawContext;drawTextWithShadow(Lnet/minecraft/client/font/TextRenderer;Lnet/minecraft/text/Text;III)I"
        ),
        index = 4
    )
    private int paraguacraft$highlightNick(int color, Text text) {
        if (!ModernConfig.nickFinderEnabled || text == null) {
            return color;
        }
        String plain = text.getString();
        if (plain.isEmpty() || plain.length() > 20 || plain.contains(" ")) {
            return color;
        }
        if (NickFinderManager.matches(plain)) {
            return UiTheme.accent();
        }
        return color;
    }
}
