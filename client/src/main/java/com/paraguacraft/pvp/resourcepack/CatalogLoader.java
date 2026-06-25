package com.paraguacraft.pvp.resourcepack;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.io.BufferedReader;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;

/** Carga el catálogo embebido y opcionalmente el remoto en GitHub. */
public final class CatalogLoader {

    private static final String USER_AGENT = "Paraguacraft-Client/2.0 (Forge-1.8.9)";
    private static final String REMOTE_CATALOG =
        "https://raw.githubusercontent.com/SantiJ10/Paraguacraft/main/clientes/paraguacraft-pvp/packs/catalog.json";
    private static final String EMBEDDED = "/assets/paraguacraft/packs/catalog.json";

    private static CatalogPack[] cached;
    private static long cachedAt;
    private static String remoteBaseUrl =
        "https://github.com/SantiJ10/Paraguacraft/releases/download/pvp-packs-1.0";
    private static String driveFolderUrl = "";

    private CatalogLoader() {}

    public static CatalogPack[] getFeatured() {
        if (cached != null && System.currentTimeMillis() - cachedAt < 300_000L) {
            return cached;
        }
        CatalogPack[] packs = loadEmbedded();
        try {
            CatalogPack[] remote = loadRemote();
            if (remote.length > 0) {
                packs = remote;
            }
        } catch (Exception ignored) {
        }
        cached = packs;
        cachedAt = System.currentTimeMillis();
        return packs;
    }

    public static String getDriveFolderUrl() {
        getFeatured();
        return driveFolderUrl;
    }

    public static void refreshAsync(final Runnable onDone) {
        new Thread(new Runnable() {
            @Override
            public void run() {
                cached = null;
                getFeatured();
                if (onDone != null) {
                    onDone.run();
                }
            }
        }, "Paraguacraft-Catalog").start();
    }

    private static CatalogPack[] loadEmbedded() {
        InputStream in = CatalogLoader.class.getResourceAsStream(EMBEDDED);
        if (in == null) {
            return new CatalogPack[0];
        }
        try {
            return parseCatalog(readStream(in));
        } catch (Exception e) {
            return new CatalogPack[0];
        }
    }

    private static CatalogPack[] loadRemote() throws Exception {
        HttpURLConnection conn = (HttpURLConnection) new URL(REMOTE_CATALOG).openConnection();
        conn.setRequestProperty("User-Agent", USER_AGENT);
        conn.setConnectTimeout(8000);
        conn.setReadTimeout(12000);
        if (conn.getResponseCode() < 200 || conn.getResponseCode() >= 300) {
            throw new IllegalStateException("HTTP " + conn.getResponseCode());
        }
        try (BufferedReader reader = new BufferedReader(
            new InputStreamReader(conn.getInputStream(), StandardCharsets.UTF_8))) {
            StringBuilder sb = new StringBuilder();
            String line;
            while ((line = reader.readLine()) != null) {
                sb.append(line);
            }
            return parseCatalog(sb.toString());
        }
    }

    private static String readStream(InputStream in) throws Exception {
        StringBuilder sb = new StringBuilder();
        try (BufferedReader reader = new BufferedReader(new InputStreamReader(in, StandardCharsets.UTF_8))) {
            String line;
            while ((line = reader.readLine()) != null) {
                sb.append(line);
            }
        }
        return sb.toString();
    }

    private static CatalogPack[] parseCatalog(String json) {
        JsonObject root = new JsonParser().parse(json).getAsJsonObject();
        if (root.has("baseUrl") && !root.get("baseUrl").isJsonNull()) {
            remoteBaseUrl = root.get("baseUrl").getAsString();
        }
        if (root.has("driveFolderUrl") && !root.get("driveFolderUrl").isJsonNull()) {
            driveFolderUrl = root.get("driveFolderUrl").getAsString();
        }
        JsonArray arr = root.getAsJsonArray("packs");
        List<CatalogPack> out = new ArrayList<CatalogPack>();
        for (JsonElement el : arr) {
            JsonObject o = el.getAsJsonObject();
            String fileName = str(o, "fileName");
            String downloadUrl = str(o, "downloadUrl");
            if ((downloadUrl == null || downloadUrl.isEmpty()) && fileName != null && !fileName.isEmpty()) {
                downloadUrl = remoteBaseUrl + "/" + fileName;
            }
            out.add(new CatalogPack(
                str(o, "id"),
                str(o, "title"),
                str(o, "subtitle"),
                downloadUrl,
                str(o, "fallbackDownloadUrl"),
                fileName,
                str(o, "sha1"),
                str(o, "badge")
            ));
        }
        return out.toArray(new CatalogPack[out.size()]);
    }

    private static String str(JsonObject o, String key) {
        if (!o.has(key) || o.get(key).isJsonNull()) {
            return null;
        }
        return o.get(key).getAsString();
    }
}
