package com.paraguacraft.pvp.core;

import com.paraguacraft.pvp.modules.ModConfig;
import java.util.Collection;
import java.util.Locale;
import net.minecraft.client.Minecraft;
import net.minecraft.scoreboard.Score;
import net.minecraft.scoreboard.ScoreObjective;
import net.minecraft.scoreboard.ScorePlayerTeam;
import net.minecraft.scoreboard.Scoreboard;
import net.minecraft.util.EnumChatFormatting;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import net.minecraftforge.fml.common.gameevent.TickEvent;

/**
 * Detecta BedWars / EggWars por scoreboard en cualquier servidor (sin depender de Hypixel)
 * y auto-activa el pack HUD si {@link ModConfig#autoBedwarsHud} está ON.
 */
public final class BedwarsModeHelper {

    private static boolean inBedwars;
    private static boolean baselineCaptured;
    private static boolean resSaved;
    private static boolean blockSaved;
    private static boolean armorSaved;
    private static boolean heldSaved;
    private static boolean potionsSaved;

    public BedwarsModeHelper() {}

    public static boolean isBedwars() {
        return inBedwars;
    }

    @SubscribeEvent
    public void onClientTick(TickEvent.ClientTickEvent event) {
        if (event.phase != TickEvent.Phase.END) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc == null || mc.theWorld == null || mc.thePlayer == null) {
            leaveBedwars();
            return;
        }
        if (mc.isIntegratedServerRunning()) {
            leaveBedwars();
            return;
        }
        boolean detected = detectFromScoreboard(mc);
        if (detected) {
            enterBedwars();
        } else {
            leaveBedwars();
        }
    }

    private static void enterBedwars() {
        if (!baselineCaptured) {
            captureBaseline();
        }
        inBedwars = true;
        if (!ModConfig.autoBedwarsHud) {
            return;
        }
        ModConfig.showBedwarsResources = true;
        ModConfig.showBlockCount = true;
        ModConfig.showArmor = true;
        ModConfig.showHeldItem = true;
        ModConfig.showPotions = true;
    }

    private static void leaveBedwars() {
        if (!inBedwars) {
            return;
        }
        inBedwars = false;
        if (!ModConfig.autoBedwarsHud || !baselineCaptured) {
            return;
        }
        ModConfig.showBedwarsResources = resSaved;
        ModConfig.showBlockCount = blockSaved;
        ModConfig.showArmor = armorSaved;
        ModConfig.showHeldItem = heldSaved;
        ModConfig.showPotions = potionsSaved;
    }

    private static void captureBaseline() {
        resSaved = ModConfig.showBedwarsResources;
        blockSaved = ModConfig.showBlockCount;
        armorSaved = ModConfig.showArmor;
        heldSaved = ModConfig.showHeldItem;
        potionsSaved = ModConfig.showPotions;
        baselineCaptured = true;
    }

    static boolean detectFromScoreboard(Minecraft mc) {
        Scoreboard board = mc.theWorld.getScoreboard();
        if (board == null) {
            return false;
        }
        ScoreObjective obj = board.getObjectiveInDisplaySlot(1);
        if (obj == null) {
            return false;
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
        return isBedwarsText(out.toString().toUpperCase(Locale.ROOT));
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

    /** Keywords BedWars / EggWars (EN + ES), alineadas con el cliente modern. */
    static boolean isBedwarsText(String t) {
        if (t == null || t.isEmpty()) {
            return false;
        }
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
}
