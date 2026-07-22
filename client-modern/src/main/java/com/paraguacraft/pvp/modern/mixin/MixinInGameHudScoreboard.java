package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.ScoreboardFilter;
import com.paraguacraft.pvp.modern.core.ServerContext;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.font.TextRenderer;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.hud.InGameHud;
import net.minecraft.client.render.RenderTickCounter;
import net.minecraft.text.Text;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyArg;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(InGameHud.class)
public class MixinInGameHudScoreboard {

    private static final String SIDEBAR_ENTRY =
        "renderScoreboardSidebar(Lnet/minecraft/client/gui/DrawContext;Lnet/minecraft/client/render/RenderTickCounter;)V";
    private static final String SIDEBAR_DRAW =
        "renderScoreboardSidebar(Lnet/minecraft/client/gui/DrawContext;Lnet/minecraft/scoreboard/ScoreboardObjective;)V";

    @Inject(method = SIDEBAR_ENTRY, at = @At("HEAD"), cancellable = true)
    private void paraguacraft$disableSidebar(DrawContext context, RenderTickCounter tickCounter, CallbackInfo ci) {
        if (!ModernConfig.scoreboardEnabled) {
            ci.cancel();
        }
    }

    /** 1.21.11 sigue usando {@code DrawContext.fill(IIIII)} dentro de {@link #SIDEBAR_DRAW}. */
    @ModifyArg(
        method = SIDEBAR_DRAW,
        at = @At(value = "INVOKE", target = "Lnet/minecraft/client/gui/DrawContext;fill(IIIII)V"),
        index = 4
    )
    private int paraguacraft$transparentBg(int color) {
        return ModernConfig.scoreboardTransparentBg ? 0 : color;
    }

    @Redirect(
        method = SIDEBAR_DRAW,
        at = @At(
            value = "INVOKE",
            target = "Lnet/minecraft/client/gui/DrawContext;drawText(Lnet/minecraft/client/font/TextRenderer;Lnet/minecraft/text/Text;IIIZ)V"
        )
    )
    private void paraguacraft$filterDraw(
        DrawContext context,
        TextRenderer tr,
        Text text,
        int x,
        int y,
        int color,
        boolean shadow
    ) {
        if (!shouldSkip(text)) {
            context.drawText(tr, text, x, y, color, shadow);
        }
    }

    private static boolean shouldSkip(Text text) {
        if (text == null) {
            return false;
        }
        String plain = ScoreboardFilter.strip(text);
        if (ModernConfig.scoreboardHideRedNumbers && ScoreboardFilter.isScoreColumnNumber(plain)) {
            return true;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        return ModernConfig.scoreboardHideStats
            && ScoreboardFilter.shouldHide(plain, ServerContext.kind(client));
    }
}
