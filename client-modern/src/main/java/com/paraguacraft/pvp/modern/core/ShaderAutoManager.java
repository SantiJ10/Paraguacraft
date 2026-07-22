package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.client.MinecraftClient;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

/** Desactiva shaders al entrar a partida competitiva; restaura al salir. */
public final class ShaderAutoManager {

    private static String savedShaderLine;

    private ShaderAutoManager() {}

    public static void onJoin(MinecraftClient client) {
        if (!ModernConfig.shaderAutoOffInMatch || client == null) {
            return;
        }
        if (!ServerContext.isCompetitive(client)) {
            return;
        }
        GameModeDetector.tick(client);
        if (!GameModeDetector.inMatch()) {
            return;
        }
        disableShaders(client);
    }

    public static void onDisconnect() {
        if (savedShaderLine == null) {
            return;
        }
        Path options = FabricLoader.getInstance().getGameDir().resolve("options.txt");
        try {
            if (!Files.isRegularFile(options)) {
                return;
            }
            List<String> lines = Files.readAllLines(options, StandardCharsets.UTF_8);
            boolean replaced = false;
            for (int i = 0; i < lines.size(); i++) {
                if (lines.get(i).startsWith("shaderPack:")) {
                    lines.set(i, savedShaderLine);
                    replaced = true;
                    break;
                }
            }
            if (!replaced) {
                lines.add(savedShaderLine);
            }
            Files.write(options, lines, StandardCharsets.UTF_8);
        } catch (IOException ignored) {
        } finally {
            savedShaderLine = null;
        }
    }

    private static void disableShaders(MinecraftClient client) {
        Path options = FabricLoader.getInstance().getGameDir().resolve("options.txt");
        Path iris = FabricLoader.getInstance().getGameDir().resolve("config/iris.properties");
        try {
            if (Files.isRegularFile(iris)) {
                List<String> lines = Files.readAllLines(iris, StandardCharsets.UTF_8);
                for (int i = 0; i < lines.size(); i++) {
                    String line = lines.get(i).trim();
                    if (line.startsWith("shaderPack=") && !line.equals("shaderPack=")) {
                        lines.set(i, "shaderPack=");
                        Files.write(iris, lines, StandardCharsets.UTF_8);
                        return;
                    }
                }
            }
        } catch (IOException ignored) {
        }
        if (!Files.isRegularFile(options)) {
            return;
        }
        try {
            List<String> lines = Files.readAllLines(options, StandardCharsets.UTF_8);
            for (int i = 0; i < lines.size(); i++) {
                String line = lines.get(i);
                if (line.startsWith("shaderPack:") && !line.equals("shaderPack:")) {
                    savedShaderLine = line;
                    lines.set(i, "shaderPack:");
                    Files.write(options, lines, StandardCharsets.UTF_8);
                    client.options.write();
                    return;
                }
            }
        } catch (IOException ignored) {
        }
    }
}
