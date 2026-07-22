package com.paraguacraft.pvp.modern.core;

import net.minecraft.client.MinecraftClient;
import net.minecraft.scoreboard.Scoreboard;
import net.minecraft.scoreboard.ScoreboardDisplaySlot;
import net.minecraft.scoreboard.ScoreboardObjective;

import java.util.regex.Matcher;
import java.util.regex.Pattern;

/** Hierro/oro/diamante/esmeralda desde inventario + fallback scoreboard (Hypixel BW). */
public final class BedwarsResourceHelper {

    public record Counts(int iron, int gold, int diamond, int emerald) {}

    private static final Pattern IRON = Pattern.compile("(?i)(?:iron|hierro|fe)\\s*[:\\-]?\\s*(\\d+)");
    private static final Pattern GOLD = Pattern.compile("(?i)(?:gold|oro|g(?:old)?)\\s*[:\\-]?\\s*(\\d+)");
    private static final Pattern DIAMOND = Pattern.compile("(?i)(?:diamond|diamante|dia)\\s*[:\\-]?\\s*(\\d+)");
    private static final Pattern EMERALD = Pattern.compile("(?i)(?:emerald|esmeralda|eme)\\s*[:\\-]?\\s*(\\d+)");

    private BedwarsResourceHelper() {}

    public static Counts resolve(MinecraftClient client) {
        Counts inv = fromInventory(client);
        Counts sb = fromScoreboard(client);
        return new Counts(
            Math.max(inv.iron, sb.iron),
            Math.max(inv.gold, sb.gold),
            Math.max(inv.diamond, sb.diamond),
            Math.max(inv.emerald, sb.emerald)
        );
    }

    private static Counts fromInventory(MinecraftClient client) {
        if (client == null || client.player == null) {
            return new Counts(0, 0, 0, 0);
        }
        int iron = 0;
        int gold = 0;
        int diamond = 0;
        int emerald = 0;
        for (int i = 0; i < client.player.getInventory().size(); i++) {
            var stack = client.player.getInventory().getStack(i);
            if (stack.isEmpty()) {
                continue;
            }
            if (stack.isOf(net.minecraft.item.Items.IRON_INGOT)) {
                iron += stack.getCount();
            } else if (stack.isOf(net.minecraft.item.Items.GOLD_INGOT)) {
                gold += stack.getCount();
            } else if (stack.isOf(net.minecraft.item.Items.DIAMOND)) {
                diamond += stack.getCount();
            } else if (stack.isOf(net.minecraft.item.Items.EMERALD)) {
                emerald += stack.getCount();
            }
        }
        return new Counts(iron, gold, diamond, emerald);
    }

    private static Counts fromScoreboard(MinecraftClient client) {
        if (client == null || client.world == null) {
            return new Counts(0, 0, 0, 0);
        }
        Scoreboard board = client.world.getScoreboard();
        ScoreboardObjective obj = board.getObjectiveForSlot(ScoreboardDisplaySlot.SIDEBAR);
        if (obj == null) {
            return new Counts(0, 0, 0, 0);
        }
        String haystack = GameModeDetector.collectSidebarText(board, obj);
        if (haystack.isEmpty()) {
            return new Counts(0, 0, 0, 0);
        }
        return new Counts(
            parseMax(haystack, IRON),
            parseMax(haystack, GOLD),
            parseMax(haystack, DIAMOND),
            parseMax(haystack, EMERALD)
        );
    }

    private static int parseMax(String text, Pattern pattern) {
        int max = 0;
        Matcher m = pattern.matcher(text);
        while (m.find()) {
            try {
                max = Math.max(max, Integer.parseInt(m.group(1)));
            } catch (NumberFormatException ignored) {
            }
        }
        return max;
    }
}
