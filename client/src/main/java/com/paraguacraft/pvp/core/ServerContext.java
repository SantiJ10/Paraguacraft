package com.paraguacraft.pvp.core;

import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ServerData;

/** Contexto de red: Hypixel, CubeCraft, Minemen, LATAM, practice. */
public final class ServerContext {

    public enum Kind {
        UNKNOWN,
        HYPIXEL,
        CUBECRAFT,
        MINEMEN,
        MUSH,
        UNIVERSOCRAFT,
        REGORLAND,
        PRACTICE,
        SINGLEPLAYER
    }

    private ServerContext() {}

    public static Kind kind() {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc == null) {
            return Kind.UNKNOWN;
        }
        if (mc.isIntegratedServerRunning()) {
            return Kind.SINGLEPLAYER;
        }
        ServerData data = mc.getCurrentServerData();
        if (data == null || data.serverIP == null) {
            return Kind.UNKNOWN;
        }
        return kindFromAddress(data.serverIP);
    }

    public static Kind kindFromAddress(String address) {
        if (address == null || address.isEmpty()) {
            return Kind.UNKNOWN;
        }
        String ip = address.toLowerCase();
        if (ip.contains("localhost") || ip.contains("127.0.0.1") || ip.startsWith("lan")) {
            return Kind.PRACTICE;
        }
        if (ip.contains("hypixel.net") || ip.contains("hypixel.io")) {
            return Kind.HYPIXEL;
        }
        if (ip.contains("cubecraft.net") || ip.contains("cubecraft")) {
            return Kind.CUBECRAFT;
        }
        if (ip.contains("minemen") || ip.contains("mineman") || ip.contains("mmc.") || ip.contains("mmc.re")) {
            return Kind.MINEMEN;
        }
        if (ip.contains("mush.com") || ip.contains("mushmc") || ip.contains("mush.")) {
            return Kind.MUSH;
        }
        if (ip.contains("universocraft") || ip.contains("uc.gg")) {
            return Kind.UNIVERSOCRAFT;
        }
        if (ip.contains("regorland") || ip.contains("librecraft")) {
            return Kind.REGORLAND;
        }
        return Kind.UNKNOWN;
    }

    public static boolean isPractice() {
        Kind k = kind();
        return k == Kind.SINGLEPLAYER || k == Kind.PRACTICE;
    }

    public static String serverLabel() {
        switch (kind()) {
            case HYPIXEL: return "Hypixel";
            case CUBECRAFT: return "CubeCraft";
            case MINEMEN: return "Minemen";
            case MUSH: return "Mush";
            case UNIVERSOCRAFT: return "UniversoCraft";
            case REGORLAND: return "Regorland";
            case SINGLEPLAYER: return "Practica";
            case PRACTICE: return "Local";
            default: {
                Minecraft mc = Minecraft.getMinecraft();
                if (mc != null && mc.getCurrentServerData() != null && mc.getCurrentServerData().serverIP != null) {
                    return mc.getCurrentServerData().serverIP;
                }
                return "Desconectado";
            }
        }
    }

    public static String chip() {
        return serverLabel() + " · " + GameModeDetector.currentLabel();
    }
}
