package com.paraguacraft.pvp.modern.core;

import net.minecraft.block.BlockState;
import net.minecraft.client.MinecraftClient;
import net.minecraft.entity.player.PlayerEntity;
import net.minecraft.registry.Registries;
import net.minecraft.scoreboard.Team;
import net.minecraft.util.DyeColor;
import net.minecraft.util.Formatting;
import net.minecraft.util.Identifier;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.Direction;
import net.minecraft.world.BlockRenderView;

/** Colores de equipo Hypixel/BedWars (camas, tab, nametags). */
public final class TeamColorHelper {

    private TeamColorHelper() {}

    /** Tinte de cama segun lana adyacente (estilo Hytils colored beds). */
    public static int getBedTint(BlockRenderView world, BlockPos pos) {
        if (world == null || pos == null) {
            return -1;
        }
        DyeColor best = null;
        int bestScore = 0;
        for (Direction dir : Direction.values()) {
            BlockState neighbor = world.getBlockState(pos.offset(dir));
            DyeColor color = woolColor(neighbor);
            if (color == null || color == DyeColor.WHITE || color == DyeColor.LIGHT_GRAY) {
                continue;
            }
            int score = dir.getAxis().isHorizontal() ? 2 : 1;
            if (score >= bestScore) {
                bestScore = score;
                best = color;
            }
        }
        if (best == null) {
            return -1;
        }
        return 0xFF000000 | best.getEntityColor();
    }

    private static DyeColor woolColor(BlockState state) {
        Identifier id = Registries.BLOCK.getId(state.getBlock());
        String path = id.getPath();
        if (!path.endsWith("_wool")) {
            return null;
        }
        String dyeName = path.substring(0, path.length() - "_wool".length());
        for (DyeColor color : DyeColor.values()) {
            if (color.name().equalsIgnoreCase(dyeName)) {
                return color;
            }
        }
        return null;
    }

    public static int getPlayerListColor(PlayerEntity player) {
        if (player == null) {
            return -1;
        }
        Team team = player.getScoreboardTeam();
        if (team == null) {
            return -1;
        }
        Formatting formatting = team.getColor();
        if (formatting == null || !formatting.isColor()) {
            return -1;
        }
        Integer rgb = formatting.getColorValue();
        return rgb == null ? -1 : (0xFF000000 | rgb);
    }

    public static int getNametagColor(PlayerEntity player) {
        return getPlayerListColor(player);
    }

    public static boolean onHypixel() {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client == null || client.getNetworkHandler() == null) {
            return false;
        }
        String addr = client.getNetworkHandler().getConnection().getAddress().toString().toLowerCase();
        return addr.contains("hypixel");
    }
}
