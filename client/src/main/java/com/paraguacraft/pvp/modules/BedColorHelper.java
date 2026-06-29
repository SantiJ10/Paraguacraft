package com.paraguacraft.pvp.modules;

import net.minecraft.block.Block;
import net.minecraft.block.BlockBed;
import net.minecraft.block.BlockColored;
import net.minecraft.block.state.IBlockState;
import net.minecraft.client.Minecraft;
import net.minecraft.init.Blocks;
import net.minecraft.item.EnumDyeColor;
import net.minecraft.scoreboard.ScorePlayerTeam;
import net.minecraft.util.BlockPos;
import net.minecraft.util.EnumChatFormatting;
import net.minecraft.world.IBlockAccess;

/** Colores de cama según equipo (BedWars / scoreboard). */
public final class BedColorHelper {

    private BedColorHelper() {}

    public static float[] getColor(IBlockAccess world, BlockPos pos) {
        float[] fromWool = colorFromNearbyWool(world, pos);
        if (fromWool != null) {
            return fromWool;
        }
        float[] fromTeam = colorFromLocalTeam();
        if (fromTeam != null) {
            return fromTeam;
        }
        return new float[] {1.0F, 1.0F, 1.0F};
    }

    private static float[] colorFromNearbyWool(IBlockAccess world, BlockPos pos) {
        for (int dx = -2; dx <= 2; dx++) {
            for (int dy = -1; dy <= 2; dy++) {
                for (int dz = -2; dz <= 2; dz++) {
                    BlockPos check = pos.add(dx, dy, dz);
                    IBlockState state = world.getBlockState(check);
                    Block block = state.getBlock();
                    if (block == Blocks.wool || block == Blocks.stained_hardened_clay) {
                        EnumDyeColor dye = (EnumDyeColor) state.getValue(BlockColored.COLOR);
                        return dyeToRgb(dye);
                    }
                }
            }
        }
        return null;
    }

    private static float[] colorFromLocalTeam() {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null || mc.theWorld == null) {
            return null;
        }
        ScorePlayerTeam team = mc.theWorld.getScoreboard().getPlayersTeam(mc.thePlayer.getName());
        if (team == null) {
            return null;
        }
        return formattingToRgb(team.getChatFormat());
    }

    public static boolean isBedBlock(IBlockState state) {
        return state != null && state.getBlock() instanceof BlockBed;
    }

    private static float[] dyeToRgb(EnumDyeColor dye) {
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

    private static float[] formattingToRgb(EnumChatFormatting fmt) {
        if (fmt == null) {
            return new float[] {1.0F, 1.0F, 1.0F};
        }
        switch (fmt) {
            case BLACK: return rgb(0, 0, 0);
            case DARK_BLUE: return rgb(0, 0, 170);
            case DARK_GREEN: return rgb(0, 170, 0);
            case DARK_AQUA: return rgb(0, 170, 170);
            case DARK_RED: return rgb(170, 0, 0);
            case DARK_PURPLE: return rgb(170, 0, 170);
            case GOLD: return rgb(255, 170, 0);
            case GRAY: return rgb(170, 170, 170);
            case DARK_GRAY: return rgb(85, 85, 85);
            case BLUE: return rgb(85, 85, 255);
            case GREEN: return rgb(85, 255, 85);
            case AQUA: return rgb(85, 255, 255);
            case RED: return rgb(255, 85, 85);
            case LIGHT_PURPLE: return rgb(255, 85, 255);
            case YELLOW: return rgb(255, 255, 85);
            case WHITE: return rgb(255, 255, 255);
            default: return new float[] {1.0F, 1.0F, 1.0F};
        }
    }

    private static float[] rgb(int r, int g, int b) {
        return new float[] {r / 255.0F, g / 255.0F, b / 255.0F};
    }
}
