package com.paraguacraft.pvp.modern.core;

import net.minecraft.text.Text;

import java.util.regex.Pattern;

/** Filtros del scoreboard Hypixel (como 1.8.9). */
public final class ScoreboardFilter {

    private static final Pattern SCORE_COLUMN = Pattern.compile("^\\d{1,2}$");

    private static final Pattern[] HIDE_LINES = new Pattern[] {
        Pattern.compile("(?i).*\\bprogreso\\s*:.*"),
        Pattern.compile("(?i).*\\btokens\\s*:.*"),
        Pattern.compile("(?i).*\\bkills?\\s+totales?\\s*:.*"),
        Pattern.compile("(?i).*\\bvictorias?\\s+totales?\\s*:.*"),
        Pattern.compile("(?i).*\\bmode\\s*:.*"),
        Pattern.compile("(?i).*\\boverall\\s+winstreak\\s*:.*"),
        Pattern.compile("(?i).*\\bmode\\s+winstreak\\s*:.*"),
        Pattern.compile("(?i).*\\bquests?\\s*:.*"),
        Pattern.compile("(?i).*\\bchallenges?\\s*:.*"),
        Pattern.compile("(?i).*\\bdaily\\s+reward\\s*:.*"),
        Pattern.compile("(?i).*\\bplay\\s+time\\s*:.*"),
        Pattern.compile("(?i).*\\brank\\s*:.*"),
        Pattern.compile("(?i).*\\blevel\\s*:.*"),
        Pattern.compile("(?i).*\\bexperience\\s*:.*"),
    };

    private static final String BLOCK_CHARS =
        "\\u2500-\\u257F\\u2580-\\u259F\\u25A0-\\u25FF\\u2B1B\\u2B1C\\u2758-\\u275A";

    private static final Pattern PROGRESS_BAR = Pattern.compile(
        "^[\\s\\d%()/.,:|\\[\\]_+\\-=*#" + BLOCK_CHARS + "]*"
            + "[" + BLOCK_CHARS + "]"
            + "[\\s\\d%()/.,:|\\[\\]_+\\-=*#" + BLOCK_CHARS + "]*$");

    private ScoreboardFilter() {}

    public static boolean isScoreColumnNumber(String text) {
        if (text == null) {
            return false;
        }
        return SCORE_COLUMN.matcher(strip(text).trim()).matches();
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

    public static String strip(Text text) {
        return text == null ? "" : strip(text.getString());
    }

    public static String strip(String text) {
        if (text == null) {
            return "";
        }
        return text.replaceAll("§.", "");
    }
}
