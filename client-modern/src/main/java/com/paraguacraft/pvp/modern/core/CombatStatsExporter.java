package com.paraguacraft.pvp.modern.core;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import net.fabricmc.loader.api.FabricLoader;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Instant;

/** Exporta estadisticas de combate al cerrar sesion. */
public final class CombatStatsExporter {

    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();

    private CombatStatsExporter() {}

    public static void exportSession() {
        if (CombatStats.hits == 0 && CombatStats.deaths == 0 && CombatStats.bestCombo == 0) {
            return;
        }
        Path dir = FabricLoader.getInstance().getGameDir().resolve("paraguacraft/stats");
        Path file = dir.resolve("session-" + Instant.now().toEpochMilli() + ".json");
        SessionStats stats = new SessionStats();
        stats.timestamp = Instant.now().toString();
        stats.hits = CombatStats.hits;
        stats.deaths = CombatStats.deaths;
        stats.bestCombo = CombatStats.bestCombo;
        stats.possibleKills = CombatStats.possibleKills;
        stats.lastReach = CombatStats.lastReach;
        stats.gameMode = GameModeDetector.currentLabel();
        stats.server = ServerContext.serverLabel(net.minecraft.client.MinecraftClient.getInstance());
        try {
            Files.createDirectories(dir);
            Files.writeString(file, GSON.toJson(stats), StandardCharsets.UTF_8);
        } catch (IOException ignored) {
        }
    }

    private static final class SessionStats {
        String timestamp;
        int hits;
        int deaths;
        int bestCombo;
        int possibleKills;
        double lastReach;
        String gameMode;
        String server;
    }
}
