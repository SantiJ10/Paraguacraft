package com.paraguacraft.pvp.modern.resourcepack;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import net.minecraft.client.MinecraftClient;

import java.io.BufferedReader;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;

public final class CatalogLoader {

    private static final String USER_AGENT = "Paraguacraft-Modern/1.0 (Fabric-1.21.11)";
    private static final String REMOTE =
        "https://raw.githubusercontent.com/SantiJ10/Paraguacraft/main/clientes/paraguacraft-pvp-modern/packs/catalog.json";
    private static final String EMBEDDED = "/assets/paraguacraft-modern/packs/catalog.json";

    private static CatalogPack[] cached;
    private static long cachedAt;
    private static String baseUrl =
        "https://github.com/SantiJ10/Paraguacraft/releases/download/pvp-packs-modern-1.0";

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

    private static CatalogPack[] loadEmbedded() {
        try (InputStream in = CatalogLoader.class.getResourceAsStream(EMBEDDED)) {
            if (in == null) {
                return new CatalogPack[0];
            }
            return parse(readStream(in));
        } catch (Exception e) {
            return new CatalogPack[0];
        }
    }

    private static CatalogPack[] loadRemote() throws Exception {
        HttpURLConnection conn = (HttpURLConnection) new URL(REMOTE).openConnection();
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
            return parse(sb.toString());
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

    private static CatalogPack[] parse(String json) {
        JsonObject root = JsonParser.parseString(json).getAsJsonObject();
        if (root.has("baseUrl")) {
            baseUrl = root.get("baseUrl").getAsString();
        }
        JsonArray arr = root.getAsJsonArray("packs");
        List<CatalogPack> out = new ArrayList<>();
        for (JsonElement el : arr) {
            JsonObject o = el.getAsJsonObject();
            String fileName = str(o, "fileName");
            String url = baseUrl + "/" + fileName;
            out.add(new CatalogPack(
                str(o, "id"),
                str(o, "title"),
                str(o, "subtitle"),
                url,
                fileName,
                str(o, "sha1"),
                str(o, "badge")
            ));
        }
        return out.toArray(CatalogPack[]::new);
    }

    private static String str(JsonObject o, String key) {
        return o.has(key) && !o.get(key).isJsonNull() ? o.get(key).getAsString() : "";
    }
}
