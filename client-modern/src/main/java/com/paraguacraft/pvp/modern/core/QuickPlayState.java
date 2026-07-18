package com.paraguacraft.pvp.modern.core;

import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.client.MinecraftClient;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Properties;

/** Recuerda el ultimo modo Hypixel Quick Play y ejecuta comandos pendientes al conectar. */
public final class QuickPlayState {

    public static String lastCommand = "";
    public static String lastLabel = "";
    public static String pendingCommand = "";

    private QuickPlayState() {}

    public static boolean hasLast() {
        return lastCommand != null && !lastCommand.isEmpty();
    }

    public static void remember(String command, String label) {
        if (command == null || command.isEmpty()) {
            return;
        }
        lastCommand = command;
        lastLabel = label != null ? label : command;
        persist();
    }

    public static void queue(String command) {
        pendingCommand = command == null ? "" : command;
    }

    public static void reconnect(MinecraftClient client) {
        if (!hasLast() || client == null) {
            return;
        }
        if (HypixelHelper.isOnHypixel(client)) {
            HypixelHelper.sendCommand(client, lastCommand);
        } else {
            queue(lastCommand);
            HypixelHelper.connect(client, client.currentScreen);
        }
    }

    public static void onJoin(MinecraftClient client) {
        if (client == null || pendingCommand == null || pendingCommand.isEmpty()) {
            return;
        }
        if (!HypixelHelper.isOnHypixel(client)) {
            return;
        }
        client.execute(() -> {
            HypixelHelper.sendCommand(client, pendingCommand);
            pendingCommand = "";
        });
    }

    public static void load() {
        Path path = propsFile();
        if (!Files.isRegularFile(path)) {
            return;
        }
        Properties props = new Properties();
        try (var in = Files.newInputStream(path)) {
            props.load(in);
            lastCommand = props.getProperty("quickPlayLastCommand", "");
            lastLabel = props.getProperty("quickPlayLastLabel", "");
        } catch (IOException ignored) {
        }
    }

    private static void persist() {
        try {
            Path path = propsFile();
            Properties props = new Properties();
            if (Files.isRegularFile(path)) {
                try (var in = Files.newInputStream(path)) {
                    props.load(in);
                }
            }
            props.setProperty("quickPlayLastCommand", lastCommand);
            props.setProperty("quickPlayLastLabel", lastLabel);
            Files.createDirectories(path.getParent());
            try (var out = Files.newOutputStream(path)) {
                props.store(out, "Paraguacraft Quick Play");
            }
        } catch (IOException ignored) {
        }
    }

    private static Path propsFile() {
        return FabricLoader.getInstance().getGameDir().resolve("paraguacraft_quickplay.properties");
    }
}
