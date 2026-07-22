package com.paraguacraft.pvp.modern.core;

import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.client.MinecraftClient;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Properties;

/** Recuerda el ultimo modo Quick Play (Hypixel / Cubecraft) y ejecuta comandos pendientes al conectar. */
public final class QuickPlayState {

    public enum TargetServer {
        HYPIXEL,
        CUBECRAFT
    }

    public static String lastCommand = "";
    public static String lastLabel = "";
    public static TargetServer lastServer = TargetServer.HYPIXEL;
    public static String pendingCommand = "";
    public static TargetServer pendingServer = TargetServer.HYPIXEL;

    private QuickPlayState() {}

    public static boolean hasLast() {
        return lastCommand != null && !lastCommand.isEmpty();
    }

    public static boolean hasLastFor(TargetServer server) {
        return hasLast() && lastServer == server;
    }

    public static void remember(TargetServer server, String command, String label) {
        if (command == null || command.isEmpty()) {
            return;
        }
        lastServer = server;
        lastCommand = command;
        lastLabel = label != null ? label : command;
        persist();
    }

    /** Compat Hypixel existente. */
    public static void remember(String command, String label) {
        remember(TargetServer.HYPIXEL, command, label);
    }

    public static void queue(TargetServer server, String command) {
        pendingServer = server;
        pendingCommand = command == null ? "" : command;
    }

    public static void queue(String command) {
        queue(TargetServer.HYPIXEL, command);
    }

    public static void reconnect(MinecraftClient client, TargetServer server) {
        if (!hasLast() || client == null) {
            return;
        }
        if (server == TargetServer.HYPIXEL && HypixelHelper.isOnHypixel(client)) {
            HypixelHelper.sendCommand(client, lastCommand);
            return;
        }
        if (server == TargetServer.CUBECRAFT && CubecraftHelper.isOnCubecraft(client)) {
            CubecraftHelper.sendCommand(client, lastCommand);
            return;
        }
        queue(server, lastCommand);
        if (server == TargetServer.CUBECRAFT) {
            CubecraftHelper.connect(client, client.currentScreen);
        } else {
            HypixelHelper.connect(client, client.currentScreen);
        }
    }

    public static void reconnect(MinecraftClient client) {
        reconnect(client, lastServer);
    }

    public static void onJoin(MinecraftClient client) {
        if (client == null || pendingCommand == null || pendingCommand.isEmpty()) {
            return;
        }
        boolean ok = switch (pendingServer) {
            case HYPIXEL -> HypixelHelper.isOnHypixel(client);
            case CUBECRAFT -> CubecraftHelper.isOnCubecraft(client);
        };
        if (!ok) {
            return;
        }
        String cmd = pendingCommand;
        pendingCommand = "";
        client.execute(() -> {
            if (pendingServer == TargetServer.CUBECRAFT) {
                CubecraftHelper.sendCommand(client, cmd);
            } else {
                HypixelHelper.sendCommand(client, cmd);
            }
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
            lastServer = parseServer(props.getProperty("quickPlayLastServer", "hypixel"));
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
            props.setProperty("quickPlayLastServer", lastServer.name().toLowerCase());
            Files.createDirectories(path.getParent());
            try (var out = Files.newOutputStream(path)) {
                props.store(out, "Paraguacraft Quick Play");
            }
        } catch (IOException ignored) {
        }
    }

    private static TargetServer parseServer(String raw) {
        if (raw != null && raw.toLowerCase().contains("cube")) {
            return TargetServer.CUBECRAFT;
        }
        return TargetServer.HYPIXEL;
    }

    private static Path propsFile() {
        return FabricLoader.getInstance().getGameDir().resolve("paraguacraft_quickplay.properties");
    }
}
