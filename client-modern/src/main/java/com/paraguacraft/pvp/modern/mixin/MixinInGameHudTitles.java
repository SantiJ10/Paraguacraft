package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.hud.InGameHud;
import net.minecraft.text.Text;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(InGameHud.class)
public class MixinInGameHudTitles {

    @Inject(method = "setTitle", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$titleToChat(Text title, CallbackInfo ci) {
        if (!ModernConfig.hideTitles || title == null || title.getString().isEmpty()) {
            return;
        }
        MinecraftClient.getInstance().inGameHud.getChatHud().addMessage(title);
        ci.cancel();
    }

    @Inject(method = "setSubtitle", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$subtitleToChat(Text subtitle, CallbackInfo ci) {
        if (!ModernConfig.hideTitles || subtitle == null || subtitle.getString().isEmpty()) {
            return;
        }
        MinecraftClient.getInstance().inGameHud.getChatHud().addMessage(subtitle);
        ci.cancel();
    }
}
