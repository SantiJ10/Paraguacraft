package com.paraguacraft.pvp.core;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiMainMenu;
import net.minecraft.world.WorldSettings;
import net.minecraft.world.WorldSettings.GameType;
import net.minecraft.world.WorldType;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import net.minecraftforge.fml.common.gameevent.TickEvent;

/**
 * Mundo flat local para practica PvP cuando el launcher activa entrenamiento.
 * Reglas PvP se aplican al entrar al mundo via comandos integrados.
 */
public final class TrainingWorldHelper {

    private static final String WORLD_NAME = "Paraguacraft Training";
    private static boolean launched;
    private static int rulesTicks;

    @SubscribeEvent
    public void onClientTick(TickEvent.ClientTickEvent event) {
        if (event.phase != TickEvent.Phase.END) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (!ModConfig.pvpTrainingAutoWorld || launched) {
            return;
        }
        if (!(mc.currentScreen instanceof GuiMainMenu) || mc.theWorld != null) {
            return;
        }
        launched = true;
        long seed = 1337L;
        WorldSettings settings = new WorldSettings(
            seed,
            GameType.SURVIVAL,
            false,
            false,
            WorldType.FLAT
        );
        mc.launchIntegratedServer(WORLD_NAME, WORLD_NAME, settings);
    }

    @SubscribeEvent
    public void onWorldTick(TickEvent.WorldTickEvent event) {
        if (event.phase != TickEvent.Phase.END || event.world.isRemote) {
            return;
        }
        if (!ModConfig.pvpTrainingAutoWorld || rulesTicks > 200) {
            return;
        }
        rulesTicks++;
        if (rulesTicks == 20) {
            event.world.getGameRules().setOrCreateGameRule("keepInventory", "true");
            event.world.getGameRules().setOrCreateGameRule("naturalRegeneration", "false");
            event.world.getGameRules().setOrCreateGameRule("doDaylightCycle", "false");
            event.world.getGameRules().setOrCreateGameRule("doMobSpawning", "false");
        }
    }

    public static void openTrainingWorld() {
        ModConfig.pvpTrainingAutoWorld = true;
        launched = false;
        rulesTicks = 0;
        Minecraft mc = Minecraft.getMinecraft();
        long seed = 1337L;
        WorldSettings settings = new WorldSettings(
            seed,
            GameType.SURVIVAL,
            false,
            false,
            WorldType.FLAT
        );
        mc.launchIntegratedServer(WORLD_NAME, WORLD_NAME, settings);
    }
}
