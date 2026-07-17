package com.paraguacraft.pvp.modern.core;

import net.minecraft.client.MinecraftClient;

/** Utilidades Hypixel (detección + envío de comandos). */
public final class HypixelHelper {

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
}
