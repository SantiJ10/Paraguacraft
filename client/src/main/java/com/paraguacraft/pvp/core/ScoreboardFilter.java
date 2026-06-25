package com.paraguacraft.pvp.core;

import java.util.regex.Pattern;
import net.minecraft.util.EnumChatFormatting;

public final class ScoreboardFilter {

    /** Columna derecha del sidebar: solo el entero del score (1–15). */
    private static final Pattern SCORE_COLUMN = Pattern.compile("^\\d{1,2}$");

    private static final Pattern[] HIDE_LINES = new Pattern[] {
        // Lobby Hypixel
        Pattern.compile("(?i).*\\bprogreso\\s*:.*"),
        Pattern.compile("(?i).*\\btokens\\s*:.*"),
        Pattern.compile("(?i).*\\bkills?\\s+totales?\\s*:.*"),
        Pattern.compile("(?i).*\\bvictorias?\\s+totales?\\s*:.*"),
        // Partida Bedwars
        Pattern.compile("(?i).*\\bmode\\s*:.*"),
        Pattern.compile("(?i).*\\boverall\\s+winstreak\\s*:.*"),
        Pattern.compile("(?i).*\\bmode\\s+winstreak\\s*:.*"),
        // Barra de progreso (solo simbolos / bloques, sin letras)
        Pattern.compile("^[\\s\\[\\]●○◆◇▪▫|_\\-+=*#]+$"),
    };

    private ScoreboardFilter() {}

    public static boolean isScoreColumnNumber(String text) {
        if (text == null) {
            return false;
        }
        String plain = EnumChatFormatting.getTextWithoutFormattingCodes(text).trim();
        return SCORE_COLUMN.matcher(plain).matches();
    }

    public static boolean shouldHide(String plainLine) {
        if (plainLine == null) {
            return false;
        }
        String t = plainLine.trim();
        if (t.isEmpty()) {
            return false;
        }
        for (Pattern p : HIDE_LINES) {
            if (p.matcher(t).matches()) {
                return true;
            }
        }
        return false;
    }
}
