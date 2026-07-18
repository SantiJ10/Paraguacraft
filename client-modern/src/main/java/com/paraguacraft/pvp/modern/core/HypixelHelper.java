package com.paraguacraft.pvp.modern.core;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.multiplayer.ConnectScreen;
import net.minecraft.client.network.ServerAddress;
import net.minecraft.client.network.ServerInfo;

/** Utilidades Hypixel (deteccion + envio de comandos + conexion). */
public final class HypixelHelper {

    public static final String HYPIXEL_ADDRESS = "mc.hypixel.net";

    private HypixelHelper() {}

    public static boolean isOnHypixel(MinecraftClient client) {
        if (client == null || client.isIntegratedServerRunning()) {
            return false;
        }
        var entry = client.getCurrentServerEntry();
        if (entry == null) {
            return false;
        }
        String ip = entry.address.toLowerCase();
        return ip.contains("hypixel.net") || ip.contains("hypixel.io");
    }

    public static void sendCommand(MinecraftClient client, String command) {
        if (client.player == null || client.getNetworkHandler() == null) {
            return;
        }
        String cmd = command.startsWith("/") ? command.substring(1) : command;
        client.getNetworkHandler().sendChatCommand(cmd);
    }

    public static void connect(MinecraftClient client, Screen parent) {
        ServerInfo info = new ServerInfo("Hypixel", HYPIXEL_ADDRESS, ServerInfo.ServerType.OTHER);
        ConnectScreen.connect(parent, client, ServerAddress.parse(HYPIXEL_ADDRESS), info, false, null);
    }
}
