package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.hud.ChatHud;
import net.minecraft.text.MutableText;
import net.minecraft.text.Style;
import net.minecraft.text.Text;
import net.minecraft.util.Formatting;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Constant;
import org.spongepowered.asm.mixin.injection.ModifyConstant;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

@Mixin(ChatHud.class)
public class MixinChatHud {

    @ModifyConstant(method = "addMessage(Lnet/minecraft/text/Text;Lnet/minecraft/network/message/MessageSignatureData;Lnet/minecraft/client/gui/hud/MessageIndicator;)V", constant = @Constant(intValue = 100), require = 0)
    private int paraguacraft$unlimitedChat(int original) {
        return ModernConfig.chatUnlimited ? 16384 : original;
    }

    @ModifyVariable(method = "addMessage(Lnet/minecraft/text/Text;)V", at = @At("HEAD"), argsOnly = true, require = 0)
    private Text paraguacraft$highlightName(Text original) {
        if (!ModernConfig.chatHighlightName || original == null) {
            return original;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.player == null) {
            return original;
        }
        String name = client.player.getName().getString();
        if (name == null || name.isEmpty()) {
            return original;
        }
        String plain = original.getString();
        if (!plain.contains(name)) {
            return original;
        }
        MutableText out = Text.empty();
        Style base = original.getStyle();
        int idx = 0;
        while (idx < plain.length()) {
            int found = plain.indexOf(name, idx);
            if (found < 0) {
                out.append(Text.literal(plain.substring(idx)).setStyle(base));
                break;
            }
            if (found > idx) {
                out.append(Text.literal(plain.substring(idx, found)).setStyle(base));
            }
            out.append(Text.literal(name).setStyle(base.withColor(Formatting.AQUA).withBold(true)));
            idx = found + name.length();
        }
        return out;
    }
}
