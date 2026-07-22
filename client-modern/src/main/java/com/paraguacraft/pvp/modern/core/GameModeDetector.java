package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.scoreboard.Scoreboard;
import net.minecraft.scoreboard.ScoreboardDisplaySlot;
import net.minecraft.scoreboard.ScoreboardEntry;
import net.minecraft.scoreboard.ScoreboardObjective;
import net.minecraft.text.Text;

import java.util.Collection;
import java.util.Locale;

/** Deduce el modo de juego desde scoreboard + servidor (Hypixel / Cubecraft). */
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
        OTHER,
        AUTO
    }

    private static Mode detected = Mode.LOBBY;
    private static Mode effective = Mode.LOBBY;
    private static String currentLabel = "Lobby";
    private static boolean manualOverride;

    private GameModeDetector() {}

    public static Mode current() {
        return effective == Mode.AUTO ? detected : effective;
    }

    public static Mode detectedMode() {
        return detected;
    }

    public static String currentLabel() {
        return currentLabel;
    }

    public static boolean isManualOverride() {
        return manualOverride;
    }

    public static boolean inMatch() {
        Mode mode = current();
        return mode != Mode.LOBBY && mode != Mode.OTHER && mode != Mode.AUTO;
    }

    public static void setManualOverride(Mode mode) {
        if (mode == null || mode == Mode.AUTO) {
            manualOverride = false;
            effective = Mode.AUTO;
            ModernConfig.gameModeOverride = "";
        } else {
            manualOverride = true;
            effective = mode;
            ModernConfig.gameModeOverride = mode.name();
        }
        ModernConfig.save();
        refreshLabel();
    }

    public static void loadOverrideFromConfig() {
        applyConfigOverride();
    }

    private static void applyConfigOverride() {
        String raw = ModernConfig.gameModeOverride;
        if (raw == null || raw.isBlank() || "AUTO".equalsIgnoreCase(raw.trim())) {
            manualOverride = false;
            effective = Mode.AUTO;
            return;
        }
        try {
            Mode mode = Mode.valueOf(raw.trim().toUpperCase(Locale.ROOT));
            if (mode == Mode.AUTO) {
                manualOverride = false;
                effective = Mode.AUTO;
            } else {
                manualOverride = true;
                effective = mode;
            }
        } catch (IllegalArgumentException ignored) {
            manualOverride = false;
            effective = Mode.AUTO;
        }
    }

    public static void tick(MinecraftClient client) {
        applyConfigOverride();
        if (client == null || client.world == null || client.player == null) {
            detected = Mode.LOBBY;
            refreshLabel();
            return;
        }
        if (!ServerContext.isCompetitive(client)) {
            if (ServerContext.isPractice(client)) {
                detected = Mode.PVP;
            } else {
                detected = Mode.LOBBY;
            }
            refreshLabel();
            return;
        }
        Scoreboard board = client.world.getScoreboard();
        ScoreboardObjective obj = board.getObjectiveForSlot(ScoreboardDisplaySlot.SIDEBAR);
        String haystack = collectSidebarText(board, obj);
        detected = detectFromText(haystack, ServerContext.kind(client));
        refreshLabel();
    }

    private static void refreshLabel() {
        Mode mode = current();
        String base = labelFor(mode);
        if (manualOverride) {
            currentLabel = base + " (manual)";
        } else if (mode == Mode.OTHER) {
            currentLabel = "Minijuego";
        } else {
            currentLabel = base;
        }
    }

    static String collectSidebarText(Scoreboard board, ScoreboardObjective obj) {
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

    static Mode detectFromText(String t, ServerContext.Kind server) {
        if (t.isEmpty()) {
            return Mode.LOBBY;
        }
        return switch (server) {
            case CUBECRAFT -> detectCubecraft(t);
            case HYPIXEL -> detectHypixel(t);
            default -> detectGeneric(t);
        };
    }

    private static Mode detectHypixel(String t) {
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

    private static Mode detectCubecraft(String t) {
        if (t.contains("EGG WAR") || t.contains("EGGWAR") || t.contains("EGG WARS")) {
            return Mode.BEDWARS;
        }
        if (t.contains("SKY WAR") || t.contains("SKYWAR")) {
            return Mode.SKYWARS;
        }
        if (t.contains("LUCKY")) {
            return Mode.LUCKY_ISLANDS;
        }
        if (t.contains("BLOCK WAR") || t.contains("BLOCKWAR")) {
            return Mode.PVP;
        }
        if (t.contains("SURVIVAL GAME") || t.contains("SURVIVAL GAMES")) {
            return Mode.PVP;
        }
        if (t.contains("DUEL")) {
            return Mode.DUELS;
        }
        if (t.contains("BUILD BATTLE") || t.contains("SPEED BUILD")) {
            return Mode.BUILD_BATTLE;
        }
        if (t.contains("TNT RUN") || t.contains("TNTRUN")) {
            return Mode.TNT_RUN;
        }
        if (t.contains("LOBBY") || t.contains("HUB") || t.contains("ONLINE") || t.contains("PLAY.CUBECRAFT")) {
            return Mode.LOBBY;
        }
        return Mode.OTHER;
    }

    private static Mode detectGeneric(String t) {
        if (t.contains("BED WAR") || t.contains("BEDWAR") || t.contains("EGG WAR")) {
            return Mode.BEDWARS;
        }
        if (t.contains("SKY WAR") || t.contains("SKYWAR")) {
            return Mode.SKYWARS;
        }
        if (t.contains("LOBBY") || t.contains("HUB")) {
            return Mode.LOBBY;
        }
        return Mode.OTHER;
    }

    public static String labelFor(Mode mode) {
        return switch (mode) {
            case BEDWARS -> "BedWars";
            case SKYWARS -> "SkyWars";
            case DUELS -> "Duels";
            case BUILD_BATTLE -> "Build Battle";
            case TNT_RUN -> "TNT Run";
            case LUCKY_ISLANDS -> "Lucky Islands";
            case PVP -> "PvP";
            case LOBBY -> "Lobby";
            case AUTO -> "Auto";
            default -> "Minijuego";
        };
    }
}
