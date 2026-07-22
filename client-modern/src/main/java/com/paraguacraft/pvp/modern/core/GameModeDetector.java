package com.paraguacraft.pvp.modern.core;

import net.minecraft.client.MinecraftClient;
import net.minecraft.scoreboard.Scoreboard;
import net.minecraft.scoreboard.ScoreboardDisplaySlot;
import net.minecraft.scoreboard.ScoreboardEntry;
import net.minecraft.scoreboard.ScoreboardObjective;
import net.minecraft.text.Text;

import java.util.Collection;
import java.util.Locale;

/** Deduce el modo de juego desde el scoreboard (Hypixel / Cubecraft). */
public final class GameModeDetector {

    public enum Mode {
        LOBBY,
        BEDWARS,
        SKYWARS,
        DUELS,
        BUILD_BATTLE,
        TNT_RUN,
        LUCKY_ISLANDS,
        PVP,
        OTHER
    }

    private static Mode current = Mode.LOBBY;
    private static String currentLabel = "Lobby";

    private GameModeDetector() {}

    public static Mode current() {
        return current;
    }

    public static String currentLabel() {
        return currentLabel;
    }

    public static boolean inMatch() {
        return current != Mode.LOBBY && current != Mode.OTHER;
    }

    public static void tick(MinecraftClient client) {
        if (client == null || client.world == null || client.player == null) {
            current = Mode.LOBBY;
            currentLabel = "Menu";
            return;
        }
        if (!ServerContext.isCompetitive(client)) {
            if (ServerContext.isPractice(client)) {
                current = Mode.PVP;
                currentLabel = "Practica";
            } else {
                current = Mode.LOBBY;
                currentLabel = "Lobby";
            }
            return;
        }
        Scoreboard board = client.world.getScoreboard();
        ScoreboardObjective obj = board.getObjectiveForSlot(ScoreboardDisplaySlot.SIDEBAR);
        String haystack = collectSidebarText(board, obj);
        Mode detected = detectFromText(haystack);
        current = detected;
        currentLabel = labelFor(detected);
    }

    private static String collectSidebarText(Scoreboard board, ScoreboardObjective obj) {
        if (obj == null) {
            return "";
        }
        StringBuilder out = new StringBuilder();
        appendText(out, obj.getDisplayName());
        Collection<ScoreboardEntry> entries = board.getScoreboardEntries(obj);
        for (ScoreboardEntry entry : entries) {
            if (entry.hidden()) {
                continue;
            }
            appendText(out, entry.display());
            appendText(out, entry.name());
        }
        return out.toString().toUpperCase(Locale.ROOT);
    }

    private static void appendText(StringBuilder out, Text text) {
        if (text == null) {
            return;
        }
        String plain = ScoreboardFilter.strip(text);
        if (plain.isEmpty()) {
            return;
        }
        if (!out.isEmpty()) {
            out.append(' ');
        }
        out.append(plain);
    }

    private static Mode detectFromText(String t) {
        if (t.isEmpty()) {
            return Mode.LOBBY;
        }
        if (t.contains("BED WAR") || t.contains("BEDWAR") || t.contains("CAMA")) {
            return Mode.BEDWARS;
        }
        if (t.contains("SKY WAR") || t.contains("SKYWAR")) {
            return Mode.SKYWARS;
        }
        if (t.contains("DUEL")) {
            return Mode.DUELS;
        }
        if (t.contains("BUILD BATTLE") || t.contains("SPEED BUILDERS")) {
            return Mode.BUILD_BATTLE;
        }
        if (t.contains("TNT RUN") || t.contains("TNTRUN") || t.contains("TNT TAG")) {
            return Mode.TNT_RUN;
        }
        if (t.contains("LUCKY")) {
            return Mode.LUCKY_ISLANDS;
        }
        if (t.contains("LOBBY") || t.contains("NETWORK LEVEL") || t.contains("ONLINE")) {
            return Mode.LOBBY;
        }
        if (t.contains("PIT") || t.contains("UHC") || t.contains("MURDER") || t.contains("ARCADE")) {
            return Mode.PVP;
        }
        return Mode.OTHER;
    }

    private static String labelFor(Mode mode) {
        return switch (mode) {
            case BEDWARS -> "BedWars";
            case SKYWARS -> "SkyWars";
            case DUELS -> "Duels";
            case BUILD_BATTLE -> "Build Battle";
            case TNT_RUN -> "TNT Run";
            case LUCKY_ISLANDS -> "Lucky Islands";
            case PVP -> "PvP";
            case LOBBY -> "Lobby";
            default -> "Minijuego";
        };
    }
}
