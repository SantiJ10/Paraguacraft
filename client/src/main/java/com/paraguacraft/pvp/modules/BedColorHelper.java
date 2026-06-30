package com.paraguacraft.pvp.modules;

import net.minecraft.block.Block;
import net.minecraft.block.BlockBed;
import net.minecraft.block.BlockColored;
import net.minecraft.block.state.IBlockState;
import net.minecraft.client.Minecraft;
import net.minecraft.client.network.NetworkPlayerInfo;
import net.minecraft.init.Blocks;
import net.minecraft.item.EnumDyeColor;
import net.minecraft.item.Item;
import net.minecraft.item.ItemStack;
import net.minecraft.scoreboard.Score;
import net.minecraft.scoreboard.ScoreObjective;
import net.minecraft.scoreboard.ScorePlayerTeam;
import net.minecraft.scoreboard.Scoreboard;
import net.minecraft.util.BlockPos;
import net.minecraft.util.EnumChatFormatting;
import net.minecraft.world.IBlockAccess;

import java.util.Collection;
import java.util.HashMap;
import java.util.Locale;
import java.util.Map;

/** Colores de cama según equipo (BedWars / Hypixel). */
public final class BedColorHelper {

    private static final Map<String, EnumDyeColor> TEAM_NAME_TO_DYE = new HashMap<String, EnumDyeColor>();

    static {
        mapTeam("rojo", "red", EnumDyeColor.RED);
        mapTeam("azul", "blue", EnumDyeColor.BLUE);
        mapTeam("verde", "green", EnumDyeColor.GREEN);
        mapTeam("amarillo", "yellow", EnumDyeColor.YELLOW);
        mapTeam("aqua", "cyan", "cian", EnumDyeColor.CYAN);
        mapTeam("blanco", "white", EnumDyeColor.WHITE);
        mapTeam("rosa", "pink", EnumDyeColor.PINK);
        mapTeam("gris", "gray", "grey", EnumDyeColor.GRAY);
        mapTeam("gris claro", "light gray", "light grey", "silver", EnumDyeColor.SILVER);
        mapTeam("naranja", "orange", EnumDyeColor.ORANGE);
        mapTeam("morado", "purple", EnumDyeColor.PURPLE);
        mapTeam("negro", "black", EnumDyeColor.BLACK);
    }

    private static EnumDyeColor cachedTeamDye = null;
    private static long cachedTeamAtMs;

    private BedColorHelper() {}

    private static void mapTeam(String a, String b, EnumDyeColor dye) {
        TEAM_NAME_TO_DYE.put(a, dye);
        TEAM_NAME_TO_DYE.put(b, dye);
    }

    private static void mapTeam(String a, String b, String c, EnumDyeColor dye) {
        mapTeam(a, b, dye);
        TEAM_NAME_TO_DYE.put(c, dye);
    }

    private static void mapTeam(String a, String b, String c, String d, EnumDyeColor dye) {
        mapTeam(a, b, c, dye);
        TEAM_NAME_TO_DYE.put(d, dye);
    }

    public static float[] getColor(IBlockAccess world, BlockPos pos) {
        float[] fromWool = colorFromNearbyTeamBlocks(world, pos);
        if (fromWool != null) {
            return fromWool;
        }
        // Cama del jugador en isla sin lana cerca: usa el color del equipo local.
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer != null && mc.thePlayer.getDistanceSq(pos) < 24.0 * 24.0) {
            float[] fromTeam = colorFromPlayerTeam();
            if (fromTeam != null) {
                return fromTeam;
            }
        }
        return new float[] {1.0F, 1.0F, 1.0F};
    }

    /** Dye del equipo local (cacheado ~1s). */
    public static EnumDyeColor getLocalTeamDye() {
        long now = System.currentTimeMillis();
        if (cachedTeamDye != null && now - cachedTeamAtMs < 1000L) {
            return cachedTeamDye;
        }
        cachedTeamDye = resolveLocalTeamDye();
        cachedTeamAtMs = now;
        return cachedTeamDye;
    }

    private static EnumDyeColor resolveLocalTeamDye() {
        EnumDyeColor fromInv = dyeFromInventoryWool();
        if (fromInv != null) {
            return fromInv;
        }
        EnumDyeColor fromSb = dyeFromHypixelScoreboard();
        if (fromSb != null) {
            return fromSb;
        }
        EnumDyeColor fromTab = dyeFromTablistTeam();
        if (fromTab != null) {
            return fromTab;
        }
        EnumDyeColor fromScoreTeam = dyeFromScoreboardTeam();
        return fromScoreTeam;
    }

    private static float[] colorFromPlayerTeam() {
        EnumDyeColor dye = getLocalTeamDye();
        return dye != null ? dyeToRgb(dye) : null;
    }

    /** Lana/arcilla/vidrio teñido cerca de la cama (radio ampliado para bases BW). */
    private static float[] colorFromNearbyTeamBlocks(IBlockAccess world, BlockPos pos) {
        for (int dx = -8; dx <= 8; dx++) {
            for (int dy = -3; dy <= 4; dy++) {
                for (int dz = -8; dz <= 8; dz++) {
                    BlockPos check = pos.add(dx, dy, dz);
                    IBlockState state = world.getBlockState(check);
                    Block block = state.getBlock();
                    if (block == Blocks.wool
                            || block == Blocks.stained_hardened_clay
                            || block == Blocks.stained_glass) {
                        EnumDyeColor dye = (EnumDyeColor) state.getValue(BlockColored.COLOR);
                        if (dye != EnumDyeColor.WHITE && dye != EnumDyeColor.SILVER) {
                            return dyeToRgb(dye);
                        }
                    }
                }
            }
        }
        return null;
    }

    /** Hypixel BW: línea del sidebar con ✓ junto al nombre del equipo (Azul ✓). */
    private static EnumDyeColor dyeFromHypixelScoreboard() {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.theWorld == null) {
            return null;
        }
        Scoreboard sb = mc.theWorld.getScoreboard();
        ScoreObjective obj = sb.getObjectiveInDisplaySlot(1);
        if (obj == null) {
            return null;
        }
        Collection<Score> scores = sb.getSortedScores(obj);
        for (Score score : scores) {
            ScorePlayerTeam team = sb.getPlayersTeam(score.getPlayerName());
            String raw = ScorePlayerTeam.formatPlayerName(team, score.getPlayerName());
            String plain = EnumChatFormatting.getTextWithoutFormattingCodes(raw).trim().toLowerCase(Locale.ROOT);
            if (plain.isEmpty()) {
                continue;
            }
            boolean mine = plain.contains("✓") || plain.contains("\u2713") || plain.contains("✔");
            if (!mine) {
                continue;
            }
            plain = plain.replace("✓", "").replace("\u2713", "").replace("✔", "")
                    .replace("✗", "").replace("\u2717", "").replace("✘", "").trim();
            EnumDyeColor byName = teamNameToDye(plain);
            if (byName != null) {
                return byName;
            }
            if (team != null) {
                EnumDyeColor byFmt = formattingToDye(team.getChatFormat());
                if (byFmt != null) {
                    return byFmt;
                }
            }
        }
        return null;
    }

    private static EnumDyeColor dyeFromTablistTeam() {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null || mc.getNetHandler() == null) {
            return null;
        }
        NetworkPlayerInfo info = mc.getNetHandler().getPlayerInfo(mc.thePlayer.getUniqueID());
        if (info == null || info.getPlayerTeam() == null) {
            return null;
        }
        return formattingToDye(info.getPlayerTeam().getChatFormat());
    }

    private static EnumDyeColor dyeFromScoreboardTeam() {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null || mc.theWorld == null) {
            return null;
        }
        ScorePlayerTeam team = mc.theWorld.getScoreboard().getPlayersTeam(mc.thePlayer.getName());
        if (team == null) {
            return null;
        }
        return formattingToDye(team.getChatFormat());
    }

    /** Lana en inventario/hotbar (prioriza la más común, típico en BW). */
    private static EnumDyeColor dyeFromInventoryWool() {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null) {
            return null;
        }
        int[] counts = new int[16];
        for (ItemStack stack : mc.thePlayer.inventory.mainInventory) {
            if (stack == null || stack.getItem() != Item.getItemFromBlock(Blocks.wool)) {
                continue;
            }
            int meta = stack.getMetadata() & 15;
            counts[meta] += stack.stackSize;
        }
        int bestMeta = -1;
        int bestCount = 0;
        for (int i = 0; i < counts.length; i++) {
            if (counts[i] > bestCount) {
                bestCount = counts[i];
                bestMeta = i;
            }
        }
        if (bestMeta < 0 || bestCount <= 0) {
            return null;
        }
        EnumDyeColor dye = EnumDyeColor.byMetadata(bestMeta);
        if (dye == EnumDyeColor.WHITE || dye == EnumDyeColor.SILVER) {
            return null;
        }
        return dye;
    }

    private static EnumDyeColor teamNameToDye(String name) {
        if (name == null) {
            return null;
        }
        String key = name.trim().toLowerCase(Locale.ROOT);
        EnumDyeColor direct = TEAM_NAME_TO_DYE.get(key);
        if (direct != null) {
            return direct;
        }
        for (Map.Entry<String, EnumDyeColor> e : TEAM_NAME_TO_DYE.entrySet()) {
            if (key.contains(e.getKey())) {
                return e.getValue();
            }
        }
        return null;
    }

    private static EnumDyeColor formattingToDye(EnumChatFormatting fmt) {
        if (fmt == null) {
            return null;
        }
        switch (fmt) {
            case RED:
            case DARK_RED:
                return EnumDyeColor.RED;
            case BLUE:
            case DARK_BLUE:
                return EnumDyeColor.BLUE;
            case GREEN:
            case DARK_GREEN:
                return EnumDyeColor.GREEN;
            case YELLOW:
            case GOLD:
                return EnumDyeColor.YELLOW;
            case AQUA:
            case DARK_AQUA:
                return EnumDyeColor.CYAN;
            case LIGHT_PURPLE:
            case DARK_PURPLE:
                return EnumDyeColor.MAGENTA;
            case WHITE:
                return EnumDyeColor.WHITE;
            case GRAY:
                return EnumDyeColor.SILVER;
            case DARK_GRAY:
                return EnumDyeColor.GRAY;
            case BLACK:
                return EnumDyeColor.BLACK;
            default:
                return null;
        }
    }

    public static boolean isBedBlock(IBlockState state) {
        return state != null && state.getBlock() instanceof BlockBed;
    }

    public static float[] dyeToRgb(EnumDyeColor dye) {
        if (dye == null) {
            return new float[] {1.0F, 1.0F, 1.0F};
        }
        switch (dye) {
            case WHITE: return rgb(255, 255, 255);
            case ORANGE: return rgb(216, 127, 51);
            case MAGENTA: return rgb(178, 76, 216);
            case LIGHT_BLUE: return rgb(102, 153, 216);
            case YELLOW: return rgb(229, 229, 51);
            case LIME: return rgb(127, 204, 25);
            case PINK: return rgb(242, 127, 165);
            case GRAY: return rgb(76, 76, 76);
            case SILVER: return rgb(153, 153, 153);
            case CYAN: return rgb(76, 127, 153);
            case PURPLE: return rgb(127, 63, 178);
            case BLUE: return rgb(51, 76, 178);
            case BROWN: return rgb(102, 76, 51);
            case GREEN: return rgb(102, 127, 51);
            case RED: return rgb(153, 51, 51);
            case BLACK: return rgb(25, 25, 25);
            default: return new float[] {1.0F, 1.0F, 1.0F};
        }
    }

    private static float[] rgb(int r, int g, int b) {
        return new float[] {r / 255.0F, g / 255.0F, b / 255.0F};
    }
}
