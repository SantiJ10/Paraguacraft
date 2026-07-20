package com.paraguacraft.pvp.core;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiMainMenu;
import net.minecraft.entity.player.EntityPlayer;
import net.minecraft.world.WorldSettings;
import net.minecraft.world.WorldSettings.GameType;
import net.minecraft.world.WorldType;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import net.minecraftforge.fml.common.gameevent.TickEvent;

import java.io.File;

/**
 * Mundo flat local para practica PvP cuando el launcher activa entrenamiento.
 * Reglas PvP y kit inicial se aplican al entrar al mundo de forma directa
 * (server-side, sin comandos de chat) para evitar problemas de hilos.
 */
public final class TrainingWorldHelper {

    private static final String WORLD_NAME = "Paraguacraft Training";
    private static boolean launched;
    private static boolean pendingKitSetup;
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
        openTrainingWorld();
    }

    @SubscribeEvent
    public void onWorldTick(TickEvent.WorldTickEvent event) {
        if (event.phase != TickEvent.Phase.END || event.world.isRemote) {
            return;
        }
        if (!ModConfig.pvpTrainingAutoWorld || rulesTicks > 60) {
            return;
        }
        rulesTicks++;
        if (rulesTicks == 20) {
            event.world.getGameRules().setOrCreateGameRule("keepInventory", "true");
            event.world.getGameRules().setOrCreateGameRule("naturalRegeneration", "false");
            event.world.getGameRules().setOrCreateGameRule("doDaylightCycle", "false");
            event.world.getGameRules().setOrCreateGameRule("doMobSpawning", "false");
        }
        if (pendingKitSetup && rulesTicks == 40 && !event.world.playerEntities.isEmpty()) {
            EntityPlayer player = (EntityPlayer) event.world.playerEntities.get(0);
            TrainingKits.giveKit(player);
            TrainingKits.placeChests(event.world, player);
            pendingKitSetup = false;
        }
    }

    public static void openTrainingWorld() {
        ModConfig.pvpTrainingAutoWorld = true;
        launched = true;
        rulesTicks = 0;
        pendingKitSetup = isFreshWorld();

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

    private static boolean isFreshWorld() {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.mcDataDir == null) {
            return false;
        }
        File levelDat = new File(new File(mc.mcDataDir, "saves/" + WORLD_NAME), "level.dat");
        return !levelDat.isFile();
    }
}
