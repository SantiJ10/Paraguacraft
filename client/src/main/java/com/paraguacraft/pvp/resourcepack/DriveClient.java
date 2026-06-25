package com.paraguacraft.pvp.resourcepack;

import java.io.BufferedReader;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.HashMap;
import java.util.Locale;
import java.util.Map;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/** Resuelve y descarga .zip desde una carpeta publica de Google Drive. */
public final class DriveClient {

    private static final String USER_AGENT = "Paraguacraft-Client/2.0 (Forge-1.8.9)";
    private static final Pattern FOLDER_ID = Pattern.compile("/folders/([a-zA-Z0-9_-]+)");
    private static final Pattern FILE_ROW = Pattern.compile(
        "aria-label=\"([^\"]+\\.zip)[^\"]*\"[^>]*ssk='5:[^:]*:([^']+)'",
        Pattern.CASE_INSENSITIVE
    );
    private static final Pattern CONFIRM = Pattern.compile("confirm=([0-9A-Za-z_]+)");

    private static Map<String, String> cachedIds;
    private static String cachedFolderId;
    private static long cachedAt;

    private DriveClient() {}

    public static String downloadUrl(String folderUrl, String fileName) throws Exception {
        String fileId = resolveFileId(folderUrl, fileName);
        return "https://drive.google.com/uc?export=download&id=" + fileId;
    }

    /** Abre la conexion final de descarga (maneja confirm token de archivos grandes). */
    public static HttpURLConnection openDownload(String folderUrl, String fileName) throws Exception {
        String fileId = resolveFileId(folderUrl, fileName);
        HttpURLConnection conn = open("https://drive.google.com/uc?export=download&id=" + fileId);
        String type = conn.getContentType();
        if (type != null && type.contains("text/html")) {
            String html = readStream(conn.getInputStream());
            conn.disconnect();
            Matcher m = CONFIRM.matcher(html);
            if (m.find()) {
                conn = open(
                    "https://drive.google.com/uc?export=download&id="
                        + fileId
                        + "&confirm="
                        + m.group(1)
                );
            }
        }
        int code = conn.getResponseCode();
        if (code < 200 || code >= 300) {
            throw new IllegalStateException("Google Drive HTTP " + code);
        }
        return conn;
    }

    private static String resolveFileId(String folderUrl, String fileName) throws Exception {
        String folderId = extractFolderId(folderUrl);
        Map<String, String> index = loadIndex(folderId, folderUrl);
        String key = normalizeName(fileName);
        String id = index.get(key);
        if (id == null) {
            throw new IllegalStateException("No se encontro " + fileName + " en Google Drive");
        }
        return id;
    }

    private static Map<String, String> loadIndex(String folderId, String folderUrl) throws Exception {
        long now = System.currentTimeMillis();
        if (cachedIds != null && folderId.equals(cachedFolderId) && now - cachedAt < 600_000L) {
            return cachedIds;
        }
        HttpURLConnection conn = open(folderUrl);
        int code = conn.getResponseCode();
        if (code < 200 || code >= 300) {
            throw new IllegalStateException("Google Drive carpeta HTTP " + code);
        }
        String html = readStream(conn.getInputStream());
        Map<String, String> index = new HashMap<String, String>();
        Matcher m = FILE_ROW.matcher(html);
        while (m.find()) {
            String name = m.group(1).trim();
            String rawId = m.group(2);
            String fileId = rawId.replaceAll("-\\d+-\\d+$", "");
            index.put(normalizeName(name), fileId);
        }
        if (index.isEmpty()) {
            throw new IllegalStateException("Carpeta de Google Drive vacia o inaccesible");
        }
        cachedIds = index;
        cachedFolderId = folderId;
        cachedAt = now;
        return index;
    }

    private static String extractFolderId(String folderUrl) {
        Matcher m = FOLDER_ID.matcher(folderUrl);
        if (!m.find()) {
            throw new IllegalArgumentException("URL de carpeta Drive invalida");
        }
        return m.group(1);
    }

    private static String normalizeName(String name) {
        return name.toLowerCase(Locale.ROOT).replace(' ', '-');
    }

    private static HttpURLConnection open(String urlStr) throws Exception {
        HttpURLConnection conn = (HttpURLConnection) new URL(urlStr).openConnection();
        conn.setRequestMethod("GET");
        conn.setConnectTimeout(20000);
        conn.setReadTimeout(120000);
        conn.setInstanceFollowRedirects(true);
        conn.setRequestProperty("User-Agent", USER_AGENT);
        return conn;
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
}
