package com.paraguacraft.pvp.core;

import net.minecraft.client.Minecraft;

/** Utilidades para detectar conexión a Hypixel. */
public final class HypixelHelper {

    private HypixelHelper() {}

    public static boolean isOnHypixel() {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.isIntegratedServerRunning() || mc.getCurrentServerData() == null) {
            return false;
        }
        String ip = mc.getCurrentServerData().serverIP.toLowerCase();
        return ip.contains("hypixel.net") || ip.contains("hypixel.io");
    }

    public static void sendCommand(String command) {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null) {
            return;
        }
        String cmd = command.startsWith("/") ? command.substring(1) : command;
        mc.thePlayer.sendChatMessage("/" + cmd);
    }
}
