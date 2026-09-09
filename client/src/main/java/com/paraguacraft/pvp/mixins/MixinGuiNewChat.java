package com.paraguacraft.pvp.mixins;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiNewChat;
import net.minecraft.util.ChatComponentText;
import net.minecraft.util.ChatStyle;
import net.minecraft.util.EnumChatFormatting;
import net.minecraft.util.IChatComponent;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Constant;
import org.spongepowered.asm.mixin.injection.ModifyConstant;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import org.spongepowered.asm.mixin.injection.Redirect;

@Mixin(GuiNewChat.class)
public class MixinGuiNewChat {

    @ModifyConstant(method = "setChatLine", constant = @Constant(intValue = 100))
    private int paraguacraft$chatHistory(int original) {
        return ModConfig.chatUnlimited ? 16384 : original;
    }

    @Redirect(
        method = "drawChat",
        at = @At(value = "INVOKE", target = "Lnet/minecraft/client/gui/FontRenderer;drawStringWithShadow(Ljava/lang/String;FFI)I")
    )
    private int paraguacraft$chatShadow(net.minecraft.client.gui.FontRenderer fr, String text, float x, float y, int color) {
        if (ModConfig.chatTextShadow) {
            return fr.drawStringWithShadow(text, x, y, color);
        }
        return fr.drawString(text, x, y, color, false);
    }

    @ModifyVariable(method = "printChatMessageWithOptionalDeletion", at = @At("HEAD"), argsOnly = true, ordinal = 0)
    private IChatComponent paraguacraft$highlightName(IChatComponent component) {
        if (!ModConfig.chatHighlightName || component == null) {
            return component;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null) {
            return component;
        }
        String name = mc.thePlayer.getName();
        if (name == null || name.isEmpty()) {
            return component;
        }
        String plain = component.getUnformattedText();
        if (plain == null || !plain.contains(name)) {
            return component;
        }
        return highlight(component, name);
    }

    private static IChatComponent highlight(IChatComponent src, String name) {
        ChatComponentText out = new ChatComponentText("");
        out.setChatStyle(copyStyle(src.getChatStyle()));
        String text = src instanceof ChatComponentText ? src.getUnformattedTextForChat() : "";
        if (text != null && !text.isEmpty() && text.contains(name)) {
            int idx = 0;
            while (idx < text.length()) {
                int found = text.indexOf(name, idx);
                if (found < 0) {
                    out.appendSibling(styled(text.substring(idx), src.getChatStyle()));
                    break;
                }
                if (found > idx) {
                    out.appendSibling(styled(text.substring(idx, found), src.getChatStyle()));
                }
                ChatStyle hi = copyStyle(src.getChatStyle());
                hi.setColor(EnumChatFormatting.AQUA);
                hi.setBold(Boolean.TRUE);
                out.appendSibling(styled(name, hi));
                idx = found + name.length();
            }
        } else if (text != null && !text.isEmpty()) {
            out.appendSibling(styled(text, src.getChatStyle()));
        }
        for (IChatComponent sib : src.getSiblings()) {
            out.appendSibling(highlight(sib, name));
        }
        return out;
    }

    private static ChatComponentText styled(String text, ChatStyle style) {
        ChatComponentText t = new ChatComponentText(text);
        t.setChatStyle(copyStyle(style));
        return t;
    }

    private static ChatStyle copyStyle(ChatStyle style) {
        ChatStyle copy = new ChatStyle();
        if (style == null) {
            return copy;
        }
        copy.setColor(style.getColor());
        copy.setBold(style.getBold());
        copy.setItalic(style.getItalic());
        copy.setUnderlined(style.getUnderlined());
        copy.setStrikethrough(style.getStrikethrough());
        copy.setObfuscated(style.getObfuscated());
        copy.setChatClickEvent(style.getChatClickEvent());
        copy.setChatHoverEvent(style.getChatHoverEvent());
        copy.setInsertion(style.getInsertion());
        return copy;
    }
}
