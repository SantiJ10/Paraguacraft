package com.paraguacraft.pvp.gui.theme;

public final class TextUtil {

    private TextUtil() {}

    /** Elimina códigos de formato § de Minecraft para CFont. */
    public static String stripColor(String input) {
        if (input == null) {
            return "";
        }
        return input.replaceAll("(?i)§.", "");
    }

    public static boolean containsIgnoreCase(String haystack, String needle) {
        if (needle == null || needle.isEmpty()) {
            return true;
        }
        return haystack != null && haystack.toLowerCase().contains(needle.toLowerCase());
    }

    /**
     * MC 1.8 solo renderiza bien ASCII en la fuente vanilla; caracteres como í (U+00ED)
     * coinciden con iconos del atlas (p. ej. ♠).
     */
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
        switch (c) {
            case '\u00C0': case '\u00C1': case '\u00C2': case '\u00C3': case '\u00C4': case '\u00C5':
                return 'A';
            case '\u00E0': case '\u00E1': case '\u00E2': case '\u00E3': case '\u00E4': case '\u00E5':
                return 'a';
            case '\u00C7': return 'C';
            case '\u00E7': return 'c';
            case '\u00C8': case '\u00C9': case '\u00CA': case '\u00CB':
                return 'E';
            case '\u00E8': case '\u00E9': case '\u00EA': case '\u00EB':
                return 'e';
            case '\u00CC': case '\u00CD': case '\u00CE': case '\u00CF':
                return 'I';
            case '\u00EC': case '\u00ED': case '\u00EE': case '\u00EF':
                return 'i';
            case '\u00D1': return 'N';
            case '\u00F1': return 'n';
            case '\u00D2': case '\u00D3': case '\u00D4': case '\u00D5': case '\u00D6':
                return 'O';
            case '\u00F2': case '\u00F3': case '\u00F4': case '\u00F5': case '\u00F6':
                return 'o';
            case '\u00D9': case '\u00DA': case '\u00DB': case '\u00DC':
                return 'U';
            case '\u00F9': case '\u00FA': case '\u00FB': case '\u00FC':
                return 'u';
            case '\u00DD': return 'Y';
            case '\u00FD': return 'y';
            default:
                if (c > 127 && c < 256) {
                    return '?';
                }
                return c;
        }
    }
}
