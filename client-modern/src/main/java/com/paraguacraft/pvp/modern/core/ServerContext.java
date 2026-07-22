package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.ServerInfo;

/** Detecta servidor conectado (Hypixel, Cubecraft, practica, etc.). */
public final class ServerContext {

    public enum Kind {
        UNKNOWN,
        HYPIXEL,
        CUBECRAFT,
        PRACTICE,
        SINGLEPLAYER
    }

    private ServerContext() {}

    public static Kind kind(MinecraftClient client) {
        if (client == null) {
            return Kind.UNKNOWN;
        }
        if (client.isIntegratedServerRunning() || client.getServer() != null) {
            return Kind.SINGLEPLAYER;
        }
        ServerInfo entry = client.getCurrentServerEntry();
        if (entry == null || entry.address == null) {
            return Kind.UNKNOWN;
        }
        String ip = entry.address.toLowerCase();
        if (ip.contains("hypixel.net") || ip.contains("hypixel.io")) {
            return Kind.HYPIXEL;
        }
        if (ip.contains("cubecraft.net") || ip.contains("cubecraft")) {
            return Kind.CUBECRAFT;
        }
        if (ip.contains("localhost") || ip.contains("127.0.0.1") || ip.contains("lan")) {
            return Kind.PRACTICE;
        }
        return Kind.UNKNOWN;
    }

    public static boolean isOnHypixel(MinecraftClient client) {
        return kind(client) == Kind.HYPIXEL;
    }

    public static boolean isOnCubecraft(MinecraftClient client) {
        return kind(client) == Kind.CUBECRAFT;
    }

    public static boolean isCompetitive(MinecraftClient client) {
        Kind k = kind(client);
        return k == Kind.HYPIXEL || k == Kind.CUBECRAFT;
    }

    public static boolean isPractice(MinecraftClient client) {
        Kind k = kind(client);
        return k == Kind.SINGLEPLAYER || k == Kind.PRACTICE;
    }

    /** Hypixel/Cubecraft bloquean freelook en ranked — respetar blacklist del cliente. */
    public static boolean freelookAllowed(MinecraftClient client) {
        if (!ModernConfig.freelookEnabled) {
            return false;
        }
        if (!ModernConfig.freelookBlacklistServers) {
            return true;
        }
        return !isCompetitive(client);
    }

    public static boolean reachDisplayAllowed(MinecraftClient client) {
        if (!ModernConfig.reachDisplay) {
            return false;
        }
        if (!ModernConfig.reachDisplayPracticeOnly) {
            return true;
        }
        return isPractice(client);
    }

    public static String serverLabel(MinecraftClient client) {
        return switch (kind(client)) {
            case HYPIXEL -> "Hypixel";
            case CUBECRAFT -> "Cubecraft";
            case SINGLEPLAYER -> "Practica";
            case PRACTICE -> "Local";
            default -> {
                ServerInfo e = client != null ? client.getCurrentServerEntry() : null;
                yield e != null && e.address != null ? e.address : "Desconocido";
            }
        };
    }
}
