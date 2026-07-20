package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.fabricmc.fabric.api.client.rendering.v1.ColorProviderRegistry;
import net.minecraft.block.BedBlock;
import net.minecraft.block.Block;
import net.minecraft.registry.Registries;

/** Camas teñidas por lana/terracota adyacente (BedWars). */
public final class ColoredBedsBootstrap {

    private ColoredBedsBootstrap() {}

    public static void register() {
        for (Block block : Registries.BLOCK) {
            if (block instanceof BedBlock) {
                ColorProviderRegistry.BLOCK.register(
                    (state, world, pos, tintIndex) -> {
                        if (!ModernConfig.coloredBeds || world == null || pos == null) {
                            return -1;
                        }
                        return TeamColorHelper.getBedTint(world, pos);
                    },
                    block
                );
            }
        }
    }
}
