package com.paraguacraft.pvp.core;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.scoreboard.Score;
import net.minecraft.scoreboard.ScoreObjective;
import net.minecraft.scoreboard.ScorePlayerTeam;
import net.minecraft.scoreboard.Scoreboard;
import net.minecraft.util.EnumChatFormatting;

import java.util.Collection;
import java.util.Locale;

/** Deduce el modo de juego desde scoreboard + servidor. */
public final class GameModeDetector {

    public enum Mode {
        LOBBY,
        BEDWARS,
        SKYWARS,
        DUELS,
        UHC,
        BUILD_BATTLE,
        TNT_RUN,
        LUCKY_ISLANDS,
        HUNGER_GAMES,
        PVP,
        OTHER,
        AUTO
    }

    private static Mode detected = Mode.LOBBY;
    private static Mode effective = Mode.AUTO;
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
            ModConfig.gameModeOverride = "";
        } else {
            manualOverride = true;
            effective = mode;
            ModConfig.gameModeOverride = mode.name();
        }
        ModConfig.save();
        refreshLabel();
    }

    public static void cycleOverride() {
        Mode[] opts = { Mode.AUTO, Mode.LOBBY, Mode.BEDWARS, Mode.SKYWARS, Mode.DUELS, Mode.UHC, Mode.PVP };
        Mode cur = manualOverride ? effective : Mode.AUTO;
        int idx = 0;
        for (int i = 0; i < opts.length; i++) {
            if (opts[i] == cur) {
                idx = i;
                break;
            }
        }
        setManualOverride(opts[(idx + 1) % opts.length]);
    }

    public static String overrideLabel() {
        if (!manualOverride) {
            return "Auto";
        }
        return labelFor(effective);
    }

    public static void loadOverrideFromConfig() {
        applyConfigOverride();
        refreshLabel();
    }

    private static void applyConfigOverride() {
        String raw = ModConfig.gameModeOverride;
        if (raw == null || raw.isEmpty() || "AUTO".equalsIgnoreCase(raw.trim())) {
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

    public static void tick() {
        applyConfigOverride();
        Minecraft mc = Minecraft.getMinecraft();
        if (mc == null || mc.theWorld == null || mc.thePlayer == null) {
            detected = Mode.LOBBY;
            refreshLabel();
            return;
        }
        if (ServerContext.isPractice()) {
            detected = Mode.PVP;
            refreshLabel();
            return;
        }
        String haystack = collectSidebarText(mc);
        detected = detectFromText(haystack, ServerContext.kind());
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

    static String collectSidebarText(Minecraft mc) {
        Scoreboard board = mc.theWorld.getScoreboard();
        if (board == null) {
            return "";
        }
        ScoreObjective obj = board.getObjectiveInDisplaySlot(1);
        if (obj == null) {
            return "";
        }
        StringBuilder out = new StringBuilder();
        appendPlain(out, obj.getDisplayName());
        Collection<Score> scores = board.getSortedScores(obj);
        for (Score score : scores) {
            if (score == null || score.getPlayerName() == null || score.getPlayerName().startsWith("#")) {
                continue;
            }
            ScorePlayerTeam team = board.getPlayersTeam(score.getPlayerName());
            String line = ScorePlayerTeam.formatPlayerName(team, score.getPlayerName());
            appendPlain(out, line);
        }
        return out.toString().toUpperCase(Locale.ROOT);
    }

    private static void appendPlain(StringBuilder out, String raw) {
        if (raw == null || raw.isEmpty()) {
            return;
        }
        String plain = EnumChatFormatting.getTextWithoutFormattingCodes(raw);
        if (plain == null || plain.isEmpty()) {
            return;
        }
        if (out.length() > 0) {
            out.append(' ');
        }
        out.append(plain);
    }

    static Mode detectFromText(String t, ServerContext.Kind server) {
        if (t == null || t.isEmpty()) {
            return Mode.LOBBY;
        }
        switch (server) {
            case CUBECRAFT:
                return detectCubecraft(t);
            case HYPIXEL:
                return detectHypixel(t);
            case MINEMEN:
                return detectMinemen(t);
            default:
                return detectGeneric(t);
        }
    }

    private static Mode detectHypixel(String t) {
        if (isBedwarsText(t)) {
            return Mode.BEDWARS;
        }
        if (t.contains("SKY WAR") || t.contains("SKYWAR")) {
            return Mode.SKYWARS;
        }
        if (t.contains("DUEL")) {
            return Mode.DUELS;
        }
        if (t.contains("UHC") || t.contains("ULTRA HARDCORE")) {
            return Mode.UHC;
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
        if (t.contains("PIT") || t.contains("MURDER") || t.contains("ARCADE")) {
            return Mode.PVP;
        }
        return Mode.OTHER;
    }

    private static Mode detectCubecraft(String t) {
        if (isBedwarsText(t)) {
            return Mode.BEDWARS;
        }
        if (t.contains("SKY WAR") || t.contains("SKYWAR")) {
            return Mode.SKYWARS;
        }
        if (t.contains("LUCKY")) {
            return Mode.LUCKY_ISLANDS;
        }
        if (t.contains("DUEL")) {
            return Mode.DUELS;
        }
        if (t.contains("SURVIVAL GAME") || t.contains("SURVIVAL GAMES")) {
            return Mode.HUNGER_GAMES;
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

    private static Mode detectMinemen(String t) {
        if (t.contains("DUEL") || t.contains("RANKED") || t.contains("UNRANKED") || t.contains("QUEUE")) {
            return Mode.DUELS;
        }
        if (t.contains("PRACTICE") || t.contains("NODEBUFF") || t.contains("POTP") || t.contains("SOUP")) {
            return Mode.DUELS;
        }
        if (t.contains("LOBBY") || t.contains("HUB") || t.contains("ONLINE")) {
            return Mode.LOBBY;
        }
        if (t.contains("ELO") || t.contains("PING") || t.contains("CPS") || t.contains("COMBO")) {
            return Mode.DUELS;
        }
        return Mode.PVP;
    }

    private static Mode detectGeneric(String t) {
        if (isBedwarsText(t)) {
            return Mode.BEDWARS;
        }
        if (t.contains("SKY WAR") || t.contains("SKYWAR") || t.contains("SW ")) {
            return Mode.SKYWARS;
        }
        if (t.contains("DUEL") || t.contains("1V1") || t.contains("2V2") || t.contains("SUMO")) {
            return Mode.DUELS;
        }
        if (t.contains("UHC") || t.contains("ULTRA HARDCORE") || t.contains("HARDCORE")) {
            return Mode.UHC;
        }
        if (t.contains("HUNGER") || t.contains("SURVIVAL GAME") || t.contains("JUEGOS DEL HAMBRE")) {
            return Mode.HUNGER_GAMES;
        }
        if (t.contains("BUILD BATTLE") || t.contains("SPEED BUILD") || t.contains("CONSTRUCTOR")) {
            return Mode.BUILD_BATTLE;
        }
        if (t.contains("TNT RUN") || t.contains("TNTRUN") || t.contains("TNT TAG")) {
            return Mode.TNT_RUN;
        }
        if (t.contains("LUCKY")) {
            return Mode.LUCKY_ISLANDS;
        }
        if (t.contains("LOBBY") || t.contains("HUB") || t.contains("SPAWN") || t.contains("SALIDA")) {
            return Mode.LOBBY;
        }
        if (t.contains("KIT PVP") || t.contains("KITPVP") || t.contains("FACTIONS") || t.contains("SURVIVAL")) {
            return Mode.PVP;
        }
        return Mode.OTHER;
    }

    static boolean isBedwarsText(String t) {
        return t.contains("BED WARS")
            || t.contains("BED WAR")
            || t.contains("BEDWARS")
            || t.contains("BEDWAR")
            || t.contains("EGG WARS")
            || t.contains("EGG WAR")
            || t.contains("EGGWARS")
            || t.contains("EGGWAR")
            || t.contains("DESTROY THE BED")
            || t.contains("DESTRUIR LA CAMA")
            || t.contains("CAMAS")
            || t.contains("CAMA");
    }

    public static String labelFor(Mode mode) {
        switch (mode) {
            case BEDWARS: return "BedWars";
            case SKYWARS: return "SkyWars";
            case DUELS: return "Duels";
            case UHC: return "UHC";
            case BUILD_BATTLE: return "Build Battle";
            case TNT_RUN: return "TNT Run";
            case LUCKY_ISLANDS: return "Lucky Islands";
            case HUNGER_GAMES: return "Hunger Games";
            case PVP: return "PvP";
            case LOBBY: return "Lobby";
            case AUTO: return "Auto";
            default: return "Minijuego";
        }
    }
}
