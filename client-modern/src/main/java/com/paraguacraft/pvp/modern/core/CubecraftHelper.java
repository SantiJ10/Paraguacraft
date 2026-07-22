package com.paraguacraft.pvp.modern.core;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.multiplayer.ConnectScreen;
import net.minecraft.client.network.ServerAddress;
import net.minecraft.client.network.ServerInfo;

/** Utilidades Cubecraft (deteccion + comandos + conexion). */
public final class CubecraftHelper {

    public static final String CUBECRAFT_ADDRESS = "play.cubecraft.net";

    private CubecraftHelper() {}

    public static boolean isOnCubecraft(MinecraftClient client) {
        return ServerContext.isOnCubecraft(client);
    }

    public static void sendCommand(MinecraftClient client, String command) {
        if (client.player == null || client.getNetworkHandler() == null) {
            return;
        }
        String cmd = command.startsWith("/") ? command.substring(1) : command;
        client.getNetworkHandler().sendChatCommand(cmd);
    }

    public static void connect(MinecraftClient client, Screen parent) {
        ServerInfo info = new ServerInfo("Cubecraft", CUBECRAFT_ADDRESS, ServerInfo.ServerType.OTHER);
        ConnectScreen.connect(parent, client, ServerAddress.parse(CUBECRAFT_ADDRESS), info, false, null);
    }
}
