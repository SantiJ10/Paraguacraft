package com.paraguacraft.pvp.modules;

import net.minecraft.util.ChatComponentText;
import net.minecraft.util.EnumChatFormatting;
import net.minecraft.util.IChatComponent;

import java.util.regex.Matcher;
import java.util.regex.Pattern;

/** Compacta mensajes repetidos del chat con multiplicador (ej: x4). */
public final class CompactChatHandler {

    private static final Pattern COUNT_SUFFIX = Pattern.compile(" x(\\d+)$");

    private CompactChatHandler() {}

    public static String plainText(IChatComponent component) {
        if (component == null) {
            return "";
        }
        return EnumChatFormatting.getTextWithoutFormattingCodes(component.getFormattedText()).trim();
    }

    public static int extractCount(String formatted) {
        Matcher m = COUNT_SUFFIX.matcher(formatted);
        if (m.find()) {
            try {
                return Integer.parseInt(m.group(1));
            } catch (NumberFormatException ignored) {
            }
        }
        return 1;
    }

    public static String stripCountSuffix(String formatted) {
        return formatted.replaceAll(" x\\d+$", "");
    }

    public static IChatComponent withCount(IChatComponent original, int count) {
        String base = stripCountSuffix(original.getFormattedText());
        if (count <= 1) {
            return original;
        }
        return new ChatComponentText(base + EnumChatFormatting.GRAY + " x" + count);
    }
}
