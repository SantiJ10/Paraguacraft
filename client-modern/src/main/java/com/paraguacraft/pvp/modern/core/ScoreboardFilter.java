package com.paraguacraft.pvp.modern.core;

import net.minecraft.text.Text;

import java.util.regex.Pattern;

/** Filtros del scoreboard por red (Hypixel / Cube / LATAM / Minemen). */
public final class ScoreboardFilter {

    private static final Pattern SCORE_COLUMN = Pattern.compile("^\\d{1,2}$");

    private static final Pattern[] HYPIXEL_HIDE_LINES = new Pattern[] {
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

    private static final Pattern[] CUBECRAFT_HIDE_LINES = new Pattern[] {
        Pattern.compile("(?i).*\\bcubecraft\\.(net|gg).*"),
        Pattern.compile("(?i).*\\bplay\\.cubecraft.*"),
        Pattern.compile("(?i).*\\bstore\\b.*"),
        Pattern.compile("(?i).*\\bwebsite\\b.*"),
        Pattern.compile("(?i).*\\bvotes?\\b.*"),
        Pattern.compile("(?i).*\\bcoins?\\b.*"),
        Pattern.compile("(?i).*\\bgems?\\b.*"),
        Pattern.compile("(?i).*\\brank\\b.*"),
        Pattern.compile("(?i).*\\blevel\\b.*"),
        Pattern.compile("(?i).*\\bexperience\\b.*"),
        Pattern.compile("(?i).*\\bplaytime\\b.*"),
        Pattern.compile("(?i).*\\bfriends?\\s+online\\b.*"),
        Pattern.compile("(?i).*\\bonline\\s*players?\\b.*"),
        Pattern.compile("(?i).*\\bhub\\b.*"),
        Pattern.compile("(?i).*\\bnews\\b.*"),
        Pattern.compile("(?i).*\\bannouncement\\b.*"),
    };

    /** Mush / UniversoCraft / Regorland y redes genéricas LATAM. */
    private static final Pattern[] LATAM_HIDE_LINES = new Pattern[] {
        Pattern.compile("(?i).*\\btienda\\b.*"),
        Pattern.compile("(?i).*\\bstore\\b.*"),
        Pattern.compile("(?i).*\\bweb(site)?\\b.*"),
        Pattern.compile("(?i).*\\bvotos?\\b.*"),
        Pattern.compile("(?i).*\\bcoins?\\b.*"),
        Pattern.compile("(?i).*\\bmonedas?\\b.*"),
        Pattern.compile("(?i).*\\brango\\b.*"),
        Pattern.compile("(?i).*\\brank\\b.*"),
        Pattern.compile("(?i).*\\bnivel\\b.*"),
        Pattern.compile("(?i).*\\blevel\\b.*"),
        Pattern.compile("(?i).*\\bdiscord\\b.*"),
        Pattern.compile("(?i).*\\bonline\\s*:.*"),
        Pattern.compile("(?i).*\\bjugadores\\s+en\\s+l[ií]nea\\b.*"),
        Pattern.compile("(?i).*\\bamigos\\s*:.*"),
        Pattern.compile("(?i).*\\bwww\\.[a-z0-9.-]+.*"),
        Pattern.compile("(?i).*\\.(net|com|br|gg)\\b.*"),
    };

    private static final Pattern[] MINEMEN_HIDE_LINES = new Pattern[] {
        Pattern.compile("(?i).*\\bstore\\b.*"),
        Pattern.compile("(?i).*\\bdiscord\\b.*"),
        Pattern.compile("(?i).*\\bminemen\\b.*"),
        Pattern.compile("(?i).*\\bwebsite\\b.*"),
        Pattern.compile("(?i).*\\bonline\\s*players?\\b.*"),
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
        return shouldHide(plainLine, ServerContext.Kind.UNKNOWN);
    }

    public static boolean shouldHide(String plainLine, ServerContext.Kind server) {
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
        for (Pattern p : patternsFor(server)) {
            if (p.matcher(t).matches()) {
                return true;
            }
        }
        // Fallback: filtros Hypixel en redes desconocidas pero no en Cube (ya cubierto).
        if (server == ServerContext.Kind.UNKNOWN || server == ServerContext.Kind.REGORLAND) {
            for (Pattern p : HYPIXEL_HIDE_LINES) {
                if (p.matcher(t).matches()) {
                    return true;
                }
            }
            for (Pattern p : LATAM_HIDE_LINES) {
                if (p.matcher(t).matches()) {
                    return true;
                }
            }
        }
        return false;
    }

    private static Pattern[] patternsFor(ServerContext.Kind server) {
        return switch (server) {
            case CUBECRAFT -> CUBECRAFT_HIDE_LINES;
            case MINEMEN -> MINEMEN_HIDE_LINES;
            case MUSH, UNIVERSOCRAFT, REGORLAND -> LATAM_HIDE_LINES;
            case HYPIXEL -> HYPIXEL_HIDE_LINES;
            default -> HYPIXEL_HIDE_LINES;
        };
    }

    public static String strip(Text text) {
        return text == null ? "" : strip(text.getString());
    }

    public static String strip(String text) {
        if (text == null) {
            return "";
        }
        StringBuilder out = new StringBuilder(text.length());
        for (int i = 0; i < text.length(); i++) {
            char c = text.charAt(i);
            if (c == '\u00A7' || c == '§') {
                if (i + 1 < text.length()) {
                    i++;
                }
                continue;
            }
            out.append(c);
        }
        return out.toString();
    }
}
