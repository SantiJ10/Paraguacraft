package com.paraguacraft.pvp.modern.core;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.ServerInfo;
import net.minecraft.server.integrated.IntegratedServer;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.util.UUID;

/** Discord Rich Presence in-game (IPC pipe). Mismo APP ID que el launcher. */
public final class DiscordPresence {

    public static final String APP_ID = "1487516329631154206";
    private static final int OP_HANDSHAKE = 0;
    private static final int OP_FRAME = 1;

    private static FileOutputStream pipeOut;
    private static FileInputStream pipeIn;
    private static boolean connected;
    private static long sessionStart = System.currentTimeMillis() / 1000L;
    private static String lastDetails = "";
    private static String lastState = "";

    private DiscordPresence() {}

    public static boolean isEnabled() {
        String off = System.getenv("PARAGUACRAFT_DISABLE_RPC");
        return off == null || off.isEmpty();
    }

    public static void connect() {
        if (!isEnabled() || connected) {
            return;
        }
        for (int i = 0; i < 10; i++) {
            File pipe = new File("\\\\.\\pipe\\discord-ipc-" + i);
            if (!pipe.exists()) {
                continue;
            }
            try {
                pipeOut = new FileOutputStream(pipe);
                pipeIn = new FileInputStream(pipe);
                writeFrame(OP_HANDSHAKE, "{\"v\":1,\"client_id\":\"" + APP_ID + "\"}");
                readReady();
                connected = true;
                sessionStart = System.currentTimeMillis() / 1000L;
                updateFromGame();
                return;
            } catch (IOException ignored) {
                closeQuietly();
            }
        }
    }

    public static void disconnect() {
        closeQuietly();
        connected = false;
        lastDetails = "";
        lastState = "";
    }

    public static void updateFromGame() {
        if (!connected) {
            return;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        if (client == null) {
            return;
        }
        String details = buildDetails(client);
        String state = buildState(client);
        if (details.equals(lastDetails) && state.equals(lastState)) {
            return;
        }
        lastDetails = details;
        lastState = state;
        long pid = getPid();
        String nonce = UUID.randomUUID().toString();
        String json = "{"
            + "\"cmd\":\"SET_ACTIVITY\","
            + "\"args\":{"
            + "\"pid\":" + pid + ","
            + "\"activity\":{"
            + "\"details\":\"" + escape(details) + "\","
            + "\"state\":\"" + escape(state) + "\","
            + "\"timestamps\":{\"start\":" + sessionStart + "},"
            + "\"assets\":{"
            + "\"large_image\":\"paraguacraft\","
            + "\"large_text\":\"Paraguacraft PvP Modern\""
            + "}"
            + "},"
            + "\"nonce\":\"" + nonce + "\""
            + "}"
            + "}";
        try {
            writeFrame(OP_FRAME, json);
        } catch (IOException ignored) {
            disconnect();
        }
    }

    private static String buildDetails(MinecraftClient client) {
        String user = envOr("PARAGUACRAFT_RPC_USER", client.getSession().getUsername());
        String mc = envOr("PARAGUACRAFT_RPC_MC", "1.21.11");
        String loader = envOr("PARAGUACRAFT_RPC_LOADER", "Fabric + Iris");
        return user + " - " + mc + " - " + loader;
    }

    private static String buildState(MinecraftClient client) {
        if (client.world == null) {
            return "En el menú";
        }
        ServerInfo server = client.getCurrentServerEntry();
        if (server != null && server.address != null && !server.address.isEmpty()) {
            return server.address;
        }
        IntegratedServer integrated = client.getServer();
        if (integrated != null) {
            String world = integrated.getSaveProperties().getLevelName();
            if (world != null && !world.isEmpty()) {
                return "Un jugador: " + world;
            }
        }
        return "Un jugador";
    }

    private static String envOr(String key, String fallback) {
        String v = System.getenv(key);
        if (v != null && !v.isBlank()) {
            return v.trim();
        }
        return fallback;
    }

    private static long getPid() {
        try {
            String name = java.lang.management.ManagementFactory.getRuntimeMXBean().getName();
            int at = name.indexOf('@');
            if (at > 0) {
                return Long.parseLong(name.substring(0, at));
            }
        } catch (Exception ignored) {
        }
        return 0L;
    }

    private static void readReady() throws IOException {
        if (pipeIn == null) {
            return;
        }
        byte[] header = new byte[8];
        int read = pipeIn.read(header);
        if (read < 8) {
            return;
        }
        ByteBuffer buf = ByteBuffer.wrap(header).order(ByteOrder.LITTLE_ENDIAN);
        buf.getInt();
        int len = buf.getInt();
        if (len > 0 && len < 65536) {
            byte[] payload = new byte[len];
            pipeIn.read(payload);
        }
    }

    private static void writeFrame(int op, String json) throws IOException {
        if (pipeOut == null) {
            return;
        }
        byte[] data = json.getBytes(StandardCharsets.UTF_8);
        ByteBuffer buf = ByteBuffer.allocate(8 + data.length).order(ByteOrder.LITTLE_ENDIAN);
        buf.putInt(op);
        buf.putInt(data.length);
        buf.put(data);
        pipeOut.write(buf.array());
        pipeOut.flush();
    }

    private static String escape(String s) {
        return s.replace("\\", "\\\\").replace("\"", "\\\"");
    }

    private static void closeQuietly() {
        try {
            if (pipeOut != null) {
                pipeOut.close();
            }
        } catch (Exception ignored) {
        }
        try {
            if (pipeIn != null) {
                pipeIn.close();
            }
        } catch (Exception ignored) {
        }
        pipeOut = null;
        pipeIn = null;
    }
}
