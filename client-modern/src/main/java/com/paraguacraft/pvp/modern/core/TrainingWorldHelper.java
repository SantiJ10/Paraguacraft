package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.gui.CustomTitleScreen;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.minecraft.client.MinecraftClient;

/** Mundo flat local para práctica PvP (launcher o botón del menú). */
public final class TrainingWorldHelper {

    private static final String WORLD_NAME = "Paraguacraft Training";

    private static boolean launched;
    private static int rulesTicks;

    private TrainingWorldHelper() {}

    public static void register() {
        ClientTickEvents.END_CLIENT_TICK.register(TrainingWorldHelper::onClientTick);
        ClientPlayConnectionEvents.JOIN.register((handler, sender, client) -> onWorldJoin(client));
    }

    private static void onClientTick(MinecraftClient client) {
        if (!ModernConfig.pvpTrainingAutoWorld || launched) {
            return;
        }
        if (client.world != null || !(client.currentScreen instanceof CustomTitleScreen)) {
            return;
        }
        openTrainingWorld(client);
    }

    private static void onWorldJoin(MinecraftClient client) {
        if (!ModernConfig.pvpTrainingAutoWorld || !client.isIntegratedServerRunning()) {
            return;
        }
        rulesTicks++;
        if (rulesTicks > 40 || client.player == null) {
            return;
        }
        if (rulesTicks == 20) {
            HypixelHelper.sendCommand(client, "gamerule keepInventory true");
            HypixelHelper.sendCommand(client, "gamerule naturalRegeneration false");
            HypixelHelper.sendCommand(client, "gamerule doDaylightCycle false");
            HypixelHelper.sendCommand(client, "gamerule doMobSpawning false");
        }
    }

    public static void openTrainingWorld(MinecraftClient client) {
        ModernConfig.pvpTrainingAutoWorld = true;
        launched = true;
        rulesTicks = 0;
        client.createIntegratedServerLoader().start(WORLD_NAME, () -> launched = false);
    }
}
