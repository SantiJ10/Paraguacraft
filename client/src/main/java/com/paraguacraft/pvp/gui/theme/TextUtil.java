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
}
