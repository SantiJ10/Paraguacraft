package com.paraguacraft.pvp.modern.gui.theme;

/** Sanitiza texto de Spotify/YouTube para la fuente vanilla de Minecraft. */
public final class TextUtil {

    private TextUtil() {}

    public static String sanitizeForMcFont(String input) {
        if (input == null || input.isEmpty()) {
            return "";
        }
        StringBuilder out = new StringBuilder(input.length());
        for (int i = 0; i < input.length(); i++) {
            char c = input.charAt(i);
            if (c == '\u00A7' && i + 1 < input.length()) {
                out.append(c).append(input.charAt(++i));
                continue;
            }
            out.append(replaceAccent(c));
        }
        return out.toString();
    }

    private static char replaceAccent(char c) {
        return switch (c) {
            case '\u00C0', '\u00C1', '\u00C2', '\u00C3', '\u00C4', '\u00C5' -> 'A';
            case '\u00E0', '\u00E1', '\u00E2', '\u00E3', '\u00E4', '\u00E5' -> 'a';
            case '\u00C7' -> 'C';
            case '\u00E7' -> 'c';
            case '\u00C8', '\u00C9', '\u00CA', '\u00CB' -> 'E';
            case '\u00E8', '\u00E9', '\u00EA', '\u00EB' -> 'e';
            case '\u00CC', '\u00CD', '\u00CE', '\u00CF' -> 'I';
            case '\u00EC', '\u00ED', '\u00EE', '\u00EF' -> 'i';
            case '\u00D1' -> 'N';
            case '\u00F1' -> 'n';
            case '\u00D2', '\u00D3', '\u00D4', '\u00D5', '\u00D6' -> 'O';
            case '\u00F2', '\u00F3', '\u00F4', '\u00F5', '\u00F6' -> 'o';
            case '\u00D9', '\u00DA', '\u00DB', '\u00DC' -> 'U';
            case '\u00F9', '\u00FA', '\u00FB', '\u00FC' -> 'u';
            case '\u00DD' -> 'Y';
            case '\u00FD' -> 'y';
            default -> (c > 127 && c < 256) ? '?' : c;
        };
    }
}
