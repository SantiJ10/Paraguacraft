package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.gui.CustomTitleScreen;
import com.paraguacraft.pvp.modern.mixin.CreateWorldScreenAccessor;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.world.CreateWorldScreen;
import net.minecraft.client.gui.screen.world.WorldCreator;
import net.minecraft.registry.entry.RegistryEntry;
import net.minecraft.world.gen.WorldPresets;
import net.minecraft.world.level.storage.LevelStorage;
import net.minecraft.world.rule.GameRules;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

/** Mundo flat local para practica PvP. */
public final class TrainingWorldHelper {

    public static final String WORLD_NAME = "Paraguacraft Training";

    private static boolean launched;
    private static boolean pendingFlatCreate;
    private static boolean pendingKitSetup;
    private static int rulesTicks;
    private static int autoCreateTicks;

    private TrainingWorldHelper() {}

    public static void register() {
        ClientTickEvents.END_CLIENT_TICK.register(TrainingWorldHelper::onClientTick);
        ClientPlayConnectionEvents.JOIN.register((handler, sender, client) -> onWorldJoin(client));
    }

    private static void onClientTick(MinecraftClient client) {
        if (pendingFlatCreate && client.currentScreen instanceof CreateWorldScreen screen) {
            autoCreateTicks++;
            if (autoCreateTicks == 2) {
                configureFlatWorld(screen);
                ((CreateWorldScreenAccessor) screen).paraguacraft$createLevel();
                pendingFlatCreate = false;
                autoCreateTicks = 0;
            }
        }

        if (!ModernConfig.pvpTrainingAutoWorld || launched) {
            return;
        }
        if (client.world != null || !(client.currentScreen instanceof CustomTitleScreen)) {
            return;
        }
        openTrainingWorld(client);
    }

    private static void configureFlatWorld(CreateWorldScreen screen) {
        CreateWorldScreenAccessor accessor = (CreateWorldScreenAccessor) screen;
        WorldCreator creator = accessor.paraguacraft$getWorldCreator();
        creator.setWorldName(WORLD_NAME);
        creator.setCheatsEnabled(true);
        for (WorldCreator.WorldType type : creator.getNormalWorldTypes()) {
            RegistryEntry<?> preset = type.preset();
            if (preset != null && preset.getKey().orElse(null) == WorldPresets.FLAT) {
                creator.setWorldType(type);
                break;
            }
        }
        GameRules rules = creator.getGameRules();
        rules.setValue(GameRules.KEEP_INVENTORY, true, null);
        rules.setValue(GameRules.NATURAL_HEALTH_REGENERATION, false, null);
        rules.setValue(GameRules.ADVANCE_TIME, false, null);
        rules.setValue(GameRules.DO_MOB_SPAWNING, false, null);
    }

    private static void onWorldJoin(MinecraftClient client) {
        if (!ModernConfig.pvpTrainingAutoWorld || !client.isIntegratedServerRunning()) {
            return;
        }
        rulesTicks++;
        if (rulesTicks > 60 || client.player == null) {
            return;
        }
        if (rulesTicks == 20) {
            HypixelHelper.sendCommand(client, "gamerule keepInventory true");
            HypixelHelper.sendCommand(client, "gamerule naturalRegeneration false");
            HypixelHelper.sendCommand(client, "gamerule doDaylightCycle false");
            HypixelHelper.sendCommand(client, "gamerule doMobSpawning false");
        }
        if (pendingKitSetup && rulesTicks == 40) {
            TrainingKits.giveKit(client);
            TrainingKits.placeChests(client);
            pendingKitSetup = false;
        }
    }

    public static void openTrainingWorld(MinecraftClient client) {
        ModernConfig.pvpTrainingAutoWorld = true;
        launched = true;
        rulesTicks = 0;

        Path saveDir = client.getLevelStorage().getSavesDirectory().resolve(WORLD_NAME);
        Path levelDat = saveDir.resolve("level.dat");
        if (Files.isDirectory(saveDir) && !Files.isRegularFile(levelDat)) {
            deleteRecursive(saveDir);
        }

        if (Files.isRegularFile(levelDat)) {
            client.createIntegratedServerLoader().start(WORLD_NAME, () -> {
                launched = false;
                client.setScreen(new CustomTitleScreen());
            });
            return;
        }

        pendingFlatCreate = true;
        pendingKitSetup = true;
        autoCreateTicks = 0;
        CreateWorldScreen.show(client, () -> {
            pendingFlatCreate = false;
            launched = false;
            client.setScreen(new CustomTitleScreen());
        });
    }

    private static void deleteRecursive(Path path) {
        try {
            if (Files.isDirectory(path)) {
                try (var stream = Files.list(path)) {
                    for (Path child : stream.toList()) {
                        deleteRecursive(child);
                    }
                }
            }
            Files.deleteIfExists(path);
        } catch (IOException ignored) {
        }
    }
}
