package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.ServerInfo;

/**
 * Contexto de red unificado: detecta servidor, política freelook / reach,
 * y etiquetas para HUD/Discord.
 */
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
        return kindFromAddress(entry.address);
    }

    public static Kind kindFromAddress(String address) {
        if (address == null || address.isBlank()) {
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
        if (ip.contains("minemen") || ip.contains("mmc.re")) {
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

    public static boolean isOnHypixel(MinecraftClient client) {
        return kind(client) == Kind.HYPIXEL;
    }

    public static boolean isOnCubecraft(MinecraftClient client) {
        return kind(client) == Kind.CUBECRAFT;
    }

    /** Redes con ranked / anticheat donde freelook y “edge” no conviene. */
    public static boolean isCompetitive(MinecraftClient client) {
        return switch (kind(client)) {
            case HYPIXEL, CUBECRAFT, MINEMEN, MUSH, UNIVERSOCRAFT -> true;
            default -> false;
        };
    }

    /** Anticheat muy estricto (Minemen) — freelook SIEMPRE off si blacklist activa. */
    public static boolean isStrictRanked(MinecraftClient client) {
        return kind(client) == Kind.MINEMEN;
    }

    public static boolean isPractice(MinecraftClient client) {
        Kind k = kind(client);
        return k == Kind.SINGLEPLAYER || k == Kind.PRACTICE;
    }

    public static boolean freelookAllowed(MinecraftClient client) {
        if (!ModernConfig.freelookEnabled) {
            return false;
        }
        if (!ModernConfig.freelookBlacklistServers) {
            return true;
        }
        if (isStrictRanked(client)) {
            return false;
        }
        return !isCompetitive(client);
    }

    public static boolean reachDisplayAllowed(MinecraftClient client) {
        if (!ModernConfig.reachDisplay) {
            return false;
        }
        if (isStrictRanked(client)) {
            return false;
        }
        if (!ModernConfig.reachDisplayPracticeOnly) {
            return true;
        }
        return isPractice(client);
    }

    /** Shader off al entrar a match en redes competitivas. */
    public static boolean shouldAutoOffShaders(MinecraftClient client) {
        return isCompetitive(client) || kind(client) == Kind.REGORLAND;
    }

    public static String serverLabel(MinecraftClient client) {
        return switch (kind(client)) {
            case HYPIXEL -> "Hypixel";
            case CUBECRAFT -> "Cubecraft";
            case MINEMEN -> "Minemen";
            case MUSH -> "Mush";
            case UNIVERSOCRAFT -> "UniversoCraft";
            case REGORLAND -> "Regorland";
            case SINGLEPLAYER -> "Practica";
            case PRACTICE -> "Local";
            default -> {
                ServerInfo e = client != null ? client.getCurrentServerEntry() : null;
                yield e != null && e.address != null ? e.address : "Desconocido";
            }
        };
    }
}
