package com.paraguacraft.pvp.modern.core;

import net.minecraft.client.MinecraftClient;

import java.util.ArrayList;
import java.util.List;

/**
 * Kit inicial + cofres pre-armados para el mundo de entrenamiento PvP.
 * Todo se aplica via comandos vanilla (permitidos para el host en singleplayer).
 */
public final class TrainingKits {

    private TrainingKits() {}

    /** Da el kit personal al jugador (espada, arco, perlas, bloques, comida). */
    public static void giveKit(MinecraftClient client) {
        String[] kit = {
            "give @s minecraft:diamond_sword 1",
            "give @s minecraft:bow 1",
            "give @s minecraft:arrow 64",
            "give @s minecraft:ender_pearl 16",
            "give @s minecraft:golden_apple 8",
            "give @s minecraft:cobblestone 64",
            "give @s minecraft:potion[potion_contents=minecraft:healing] 3",
        };
        for (String cmd : kit) {
            HypixelHelper.sendCommand(client, cmd);
        }
    }

    /** Coloca 3 cofres cerca del jugador con sets pre-armados (vanilla, pot, UHC). */
    public static void placeChests(MinecraftClient client) {
        for (String cmd : buildChestCommands()) {
            HypixelHelper.sendCommand(client, cmd);
        }
    }

    private static List<String> buildChestCommands() {
        List<String> cmds = new ArrayList<>();

        cmds.add("setblock ~3 ~ ~0 minecraft:chest");
        fillContainer(cmds, "~3 ~ ~0",
            "minecraft:diamond_helmet 1",
            "minecraft:diamond_chestplate 1",
            "minecraft:diamond_leggings 1",
            "minecraft:diamond_boots 1",
            "minecraft:diamond_sword 1",
            "minecraft:bow 1",
            "minecraft:arrow 64");

        cmds.add("setblock ~3 ~ ~2 minecraft:chest");
        fillContainer(cmds, "~3 ~ ~2",
            "minecraft:potion[potion_contents=minecraft:healing] 6",
            "minecraft:splash_potion[potion_contents=minecraft:swiftness] 3",
            "minecraft:ender_pearl 16",
            "minecraft:golden_apple 6",
            "minecraft:leather_chestplate 1");

        cmds.add("setblock ~3 ~ ~4 minecraft:chest");
        fillContainer(cmds, "~3 ~ ~4",
            "minecraft:iron_helmet 1",
            "minecraft:iron_chestplate 1",
            "minecraft:iron_leggings 1",
            "minecraft:iron_boots 1",
            "minecraft:iron_sword 1",
            "minecraft:bow 1",
            "minecraft:arrow 32",
            "minecraft:golden_apple 8",
            "minecraft:ender_pearl 8");

        return cmds;
    }

    private static void fillContainer(List<String> cmds, String pos, String... items) {
        for (int slot = 0; slot < items.length; slot++) {
            cmds.add("item replace block " + pos + " container." + slot + " with " + items[slot]);
        }
    }
}
