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
    };

    // Rangos Unicode de los glifos que Hypixel usa para barras de progreso:
    // Box Drawing (2500-257F), Block Elements (2580-259F), Geometric Shapes (25A0-25FF),
    // cuadrados grandes (2B1B/2B1C) y barras verticales pesadas (2758-275A).
    private static final String BLOCK_CHARS =
        "\\u2500-\\u257F\\u2580-\\u259F\\u25A0-\\u25FF\\u2B1B\\u2B1C\\u2758-\\u275A";

    // Linea compuesta SOLO por bloques/simbolos de barra (+ espacios, digitos, %, separadores),
    // con al menos un glifo de bloque. Atrapa "■■■□□ 60%", cuadrados cian sueltos, etc.
    private static final Pattern PROGRESS_BAR = Pattern.compile(
        "^[\\s\\d%()/.,:|\\[\\]_+\\-=*#" + BLOCK_CHARS + "]*"
        + "[" + BLOCK_CHARS + "]"
        + "[\\s\\d%()/.,:|\\[\\]_+\\-=*#" + BLOCK_CHARS + "]*$");

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
        if (PROGRESS_BAR.matcher(t).matches()) {
            return true;
        }
        for (Pattern p : HIDE_LINES) {
            if (p.matcher(t).matches()) {
                return true;
            }
        }
        return false;
    }
}
