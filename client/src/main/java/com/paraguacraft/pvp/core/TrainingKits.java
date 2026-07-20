package com.paraguacraft.pvp.core;

import net.minecraft.block.state.IBlockState;
import net.minecraft.entity.player.EntityPlayer;
import net.minecraft.init.Blocks;
import net.minecraft.init.Items;
import net.minecraft.item.ItemStack;
import net.minecraft.tileentity.TileEntity;
import net.minecraft.tileentity.TileEntityChest;
import net.minecraft.util.BlockPos;
import net.minecraft.world.World;

/**
 * Kit inicial + cofres pre-armados para el mundo de entrenamiento PvP.
 * Todo se aplica de forma directa server-side (sin comandos de chat) para
 * evitar problemas de hilos entre el server integrado y el cliente.
 */
public final class TrainingKits {

    private TrainingKits() {}

    /** Da el kit personal al jugador (espada, arco, perlas, bloques, comida). */
    public static void giveKit(EntityPlayer player) {
        give(player, new ItemStack(Items.diamond_sword));
        give(player, new ItemStack(Items.bow));
        give(player, new ItemStack(Items.arrow, 64));
        give(player, new ItemStack(Items.ender_pearl, 16));
        give(player, new ItemStack(Items.golden_apple, 8));
        give(player, new ItemStack(Blocks.cobblestone, 64));
    }

    /** Coloca 3 cofres cerca del jugador con sets pre-armados (vanilla, pot, UHC). */
    public static void placeChests(World world, EntityPlayer player) {
        BlockPos base = player.getPosition();

        placeChest(world, base.add(3, 0, 0),
            new ItemStack(Items.diamond_helmet),
            new ItemStack(Items.diamond_chestplate),
            new ItemStack(Items.diamond_leggings),
            new ItemStack(Items.diamond_boots),
            new ItemStack(Items.diamond_sword),
            new ItemStack(Items.bow),
            new ItemStack(Items.arrow, 64));

        placeChest(world, base.add(3, 0, 2),
            new ItemStack(Items.golden_apple, 8),
            new ItemStack(Items.ender_pearl, 16),
            new ItemStack(Items.leather_chestplate),
            new ItemStack(Items.speckled_melon, 8));

        placeChest(world, base.add(3, 0, 4),
            new ItemStack(Items.iron_helmet),
            new ItemStack(Items.iron_chestplate),
            new ItemStack(Items.iron_leggings),
            new ItemStack(Items.iron_boots),
            new ItemStack(Items.iron_sword),
            new ItemStack(Items.bow),
            new ItemStack(Items.arrow, 32),
            new ItemStack(Items.golden_apple, 8),
            new ItemStack(Items.ender_pearl, 8));
    }

    private static void give(EntityPlayer player, ItemStack stack) {
        if (!player.inventory.addItemStackToInventory(stack)) {
            player.dropPlayerItemWithRandomChoice(stack, false);
        }
    }

    private static void placeChest(World world, BlockPos pos, ItemStack... items) {
        IBlockState chestState = Blocks.chest.getDefaultState();
        world.setBlockState(pos, chestState);
        TileEntity tile = world.getTileEntity(pos);
        if (!(tile instanceof TileEntityChest)) {
            return;
        }
        TileEntityChest chest = (TileEntityChest) tile;
        for (int slot = 0; slot < items.length && slot < chest.getSizeInventory(); slot++) {
            chest.setInventorySlotContents(slot, items[slot]);
        }
    }
}
