package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.client.MinecraftClient;
import net.minecraft.text.Text;

import java.io.IOException;
import java.lang.reflect.Method;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

/** Desactiva shaders al entrar a partida competitiva; restaura al salir. */
public final class ShaderAutoManager {

    private static String savedShaderLine;
    private static boolean disabledThisSession;

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
        if (disableShaders(client)) {
            notifyPlayer(client, "Shaders desactivados para la partida");
        }
    }

    public static void onDisconnect() {
        if (savedShaderLine == null) {
            disabledThisSession = false;
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
            MinecraftClient client = MinecraftClient.getInstance();
            if (client != null) {
                client.options.write();
                reloadShaders(client);
                notifyPlayer(client, "Shaders restaurados");
            }
        } catch (IOException ignored) {
        } finally {
            savedShaderLine = null;
            disabledThisSession = false;
        }
    }

    private static boolean disableShaders(MinecraftClient client) {
        if (disabledThisSession) {
            return false;
        }
        Path options = FabricLoader.getInstance().getGameDir().resolve("options.txt");
        Path iris = FabricLoader.getInstance().getGameDir().resolve("config/iris.properties");
        try {
            if (Files.isRegularFile(iris)) {
                List<String> lines = Files.readAllLines(iris, StandardCharsets.UTF_8);
                for (int i = 0; i < lines.size(); i++) {
                    String line = lines.get(i).trim();
                    if (line.startsWith("shaderPack=") && !line.equals("shaderPack=")) {
                        savedShaderLine = lines.get(i);
                        lines.set(i, "shaderPack=");
                        Files.write(iris, lines, StandardCharsets.UTF_8);
                        reloadShaders(client);
                        disabledThisSession = true;
                        return true;
                    }
                }
            }
        } catch (IOException ignored) {
        }
        if (!Files.isRegularFile(options)) {
            return false;
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
                    reloadShaders(client);
                    disabledThisSession = true;
                    return true;
                }
            }
        } catch (IOException ignored) {
        }
        return false;
    }

    private static void reloadShaders(MinecraftClient client) {
        if (client == null) {
            return;
        }
        if (FabricLoader.getInstance().isModLoaded("iris")) {
            try {
                Class<?> apiClass = Class.forName("net.irisshaders.iris.api.v0.IrisApi");
                Object api = apiClass.getMethod("getInstance").invoke(null);
                for (String methodName : new String[] {"reload", "loadShaderpack"}) {
                    try {
                        Method method = apiClass.getMethod(methodName);
                        method.invoke(api);
                        return;
                    } catch (NoSuchMethodException ignored) {
                    }
                }
                Object config = apiClass.getMethod("getIrisConfig").invoke(api);
                if (config != null) {
                    Method load = config.getClass().getMethod("loadShaderpack");
                    load.invoke(config);
                }
            } catch (ReflectiveOperationException ignored) {
            }
        }
        client.options.write();
    }

    private static void notifyPlayer(MinecraftClient client, String message) {
        if (client.player != null) {
            client.player.sendMessage(Text.literal(message), true);
        }
    }
}
