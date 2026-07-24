package com.paraguacraft.pvp.modern.core;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.ServerInfo;
import net.minecraft.client.option.ServerList;

import java.io.ByteArrayOutputStream;
import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URI;
import java.net.URL;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/**
 * Si el ping no trae favicon (placeholder azul/unknown), descarga el icono real
 * desde mcsrvstat.us (mismo enfoque que usamos en la web/launcher).
 */
public final class ServerIconBootstrap {

    private static final ExecutorService POOL = Executors.newFixedThreadPool(2, r -> {
        Thread t = new Thread(r, "paraguacraft-server-icon");
        t.setDaemon(true);
        return t;
    });

    private static final Map<String, Boolean> IN_FLIGHT = new ConcurrentHashMap<>();
    private static final Map<String, Boolean> FAILED = new ConcurrentHashMap<>();

    private ServerIconBootstrap() {}

    public static void register() {
        // Se dispara desde el mixin de MultiplayerScreen / tick del cliente al abrir lista.
    }

    public static void ensureIcons(MinecraftClient client) {
        if (client == null) {
            return;
        }
        try {
            ServerList list = new ServerList(client);
            list.loadFile();
            int n = list.size();
            for (int i = 0; i < n; i++) {
                ServerInfo info = list.get(i);
                if (info == null) {
                    continue;
                }
                byte[] fav = info.getFavicon();
                if (fav != null && fav.length > 64) {
                    continue;
                }
                requestIcon(info, list);
            }
        } catch (Throwable ignored) {
        }
    }

    public static void ensureIcon(ServerInfo info) {
        if (info == null) {
            return;
        }
        byte[] fav = info.getFavicon();
        if (fav != null && fav.length > 64) {
            return;
        }
        requestIcon(info, null);
    }

    private static void requestIcon(ServerInfo info, ServerList listToSave) {
        String addr = info.address;
        if (addr == null || addr.isBlank()) {
            return;
        }
        String key = addr.toLowerCase();
        if (FAILED.containsKey(key) || IN_FLIGHT.putIfAbsent(key, Boolean.TRUE) != null) {
            return;
        }
        POOL.execute(() -> {
            try {
                byte[] png = downloadIcon(addr);
                if (png == null || png.length < 64) {
                    FAILED.put(key, Boolean.TRUE);
                    return;
                }
                byte[] validated = ServerInfo.validateFavicon(png);
                if (validated == null) {
                    FAILED.put(key, Boolean.TRUE);
                    return;
                }
                MinecraftClient client = MinecraftClient.getInstance();
                client.execute(() -> {
                    info.setFavicon(validated);
                    if (listToSave != null) {
                        try {
                            listToSave.saveFile();
                        } catch (Throwable ignored) {
                        }
                    }
                });
            } catch (Throwable t) {
                FAILED.put(key, Boolean.TRUE);
            } finally {
                IN_FLIGHT.remove(key);
            }
        });
    }

    private static byte[] downloadIcon(String address) throws Exception {
        String host = address;
        int slash = host.indexOf('/');
        if (slash >= 0) {
            host = host.substring(0, slash);
        }
        // api.mcsrvstat.us/icon/{ip} devuelve PNG 64x64
        URL url = URI.create("https://api.mcsrvstat.us/icon/" + host).toURL();
        HttpURLConnection conn = (HttpURLConnection) url.openConnection();
        conn.setConnectTimeout(4000);
        conn.setReadTimeout(6000);
        conn.setRequestProperty("User-Agent", "ParaguacraftPvP/1.0");
        conn.setInstanceFollowRedirects(true);
        try (InputStream in = conn.getInputStream(); ByteArrayOutputStream out = new ByteArrayOutputStream()) {
            byte[] buf = new byte[4096];
            int n;
            int total = 0;
            while ((n = in.read(buf)) >= 0) {
                total += n;
                if (total > 200_000) {
                    return null;
                }
                out.write(buf, 0, n);
            }
            return out.toByteArray();
        } finally {
            conn.disconnect();
        }
    }
}
