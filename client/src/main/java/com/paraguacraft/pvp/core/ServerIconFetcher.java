package com.paraguacraft.pvp.core;

import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ServerData;
import net.minecraft.client.multiplayer.ServerList;
import org.apache.commons.codec.binary.Base64;

import java.io.ByteArrayOutputStream;
import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/**
 * Rellena iconos de la lista multiplayer cuando el ping no trae favicon.
 */
public final class ServerIconFetcher {

    private static final ExecutorService POOL = Executors.newFixedThreadPool(2, new java.util.concurrent.ThreadFactory() {
        public Thread newThread(Runnable r) {
            Thread t = new Thread(r, "paraguacraft-server-icon");
            t.setDaemon(true);
            return t;
        }
    });

    private static final Map<String, Boolean> IN_FLIGHT = new ConcurrentHashMap<String, Boolean>();
    private static final Map<String, Boolean> FAILED = new ConcurrentHashMap<String, Boolean>();

    private ServerIconFetcher() {}

    public static void ensureIcons() {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc == null) {
            return;
        }
        try {
            ServerList list = new ServerList(mc);
            list.loadServerList();
            for (int i = 0; i < list.countServers(); i++) {
                ServerData data = list.getServerData(i);
                if (data == null) {
                    continue;
                }
                String icon = data.getBase64EncodedIconData();
                if (icon != null && icon.length() > 64) {
                    continue;
                }
                request(data, list);
            }
        } catch (Throwable ignored) {
        }
    }

    private static void request(final ServerData data, final ServerList list) {
        final String addr = data.serverIP;
        if (addr == null || addr.isEmpty()) {
            return;
        }
        final String key = addr.toLowerCase();
        if (FAILED.containsKey(key) || IN_FLIGHT.putIfAbsent(key, Boolean.TRUE) != null) {
            return;
        }
        POOL.execute(new Runnable() {
            public void run() {
                try {
                    byte[] png = downloadIcon(addr);
                    if (png == null || png.length < 64) {
                        FAILED.put(key, Boolean.TRUE);
                        return;
                    }
                    final String b64 = Base64.encodeBase64String(png);
                    Minecraft.getMinecraft().addScheduledTask(new Runnable() {
                        public void run() {
                            data.setBase64EncodedIconData(b64);
                            try {
                                list.saveServerList();
                            } catch (Throwable ignored) {
                            }
                        }
                    });
                } catch (Throwable t) {
                    FAILED.put(key, Boolean.TRUE);
                } finally {
                    IN_FLIGHT.remove(key);
                }
            }
        });
    }

    private static byte[] downloadIcon(String address) throws Exception {
        String host = address;
        int slash = host.indexOf('/');
        if (slash >= 0) {
            host = host.substring(0, slash);
        }
        URL url = new URL("https://api.mcsrvstat.us/icon/" + host);
        HttpURLConnection conn = (HttpURLConnection) url.openConnection();
        conn.setConnectTimeout(4000);
        conn.setReadTimeout(6000);
        conn.setRequestProperty("User-Agent", "ParaguacraftPvP/1.0");
        conn.setInstanceFollowRedirects(true);
        InputStream in = null;
        try {
            in = conn.getInputStream();
            ByteArrayOutputStream out = new ByteArrayOutputStream();
            byte[] buf = new byte[4096];
            int n;
            int total = 0;
            while ((n = in.read(buf)) >= 0) {
                total += n;
                if (total > 200000) {
                    return null;
                }
                out.write(buf, 0, n);
            }
            return out.toByteArray();
        } finally {
            if (in != null) {
                try {
                    in.close();
                } catch (Throwable ignored) {
                }
            }
            conn.disconnect();
        }
    }
}
