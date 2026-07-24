package com.paraguacraft.pvp.hud;

import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.texture.DynamicTexture;
import net.minecraft.util.ResourceLocation;

import javax.imageio.ImageIO;
import java.awt.image.BufferedImage;
import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URI;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.security.MessageDigest;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * Descarga y cachea la carátula de Spotify/YouTube para el HUD de música.
 * Prioriza la cache del launcher ({@code %APPDATA%/ParaguacraftLauncher/music-art/{sha1}.jpg}).
 */
public final class MusicArtCache {

    private static String cachedUrl = "";
    private static ResourceLocation texture;
    private static int texW = 64;
    private static int texH = 64;
    private static final AtomicBoolean loading = new AtomicBoolean(false);

    private MusicArtCache() {}

    public static ResourceLocation get(String url) {
        if (url == null || url.isEmpty()) {
            return null;
        }
        String normalized = normalizeUrl(url);
        if (normalized.isEmpty()) {
            return null;
        }
        if (!normalized.equals(cachedUrl)) {
            cachedUrl = normalized;
            texture = null;
            texW = 64;
            texH = 64;
            loading.set(false);
        }
        if (texture != null) {
            return texture;
        }
        byte[] cached = readCachedBytes(normalized);
        if (cached != null && cached.length > 0) {
            registerFromBytes(normalized, cached);
            if (texture != null) {
                return texture;
            }
        }
        if (loading.compareAndSet(false, true)) {
            final String fetchUrl = normalized;
            Thread t = new Thread(new Runnable() {
                @Override
                public void run() {
                    download(fetchUrl);
                }
            }, "Paraguacraft-MusicArt");
            t.setDaemon(true);
            t.start();
        }
        return null;
    }

    public static int getTexWidth() {
        return texW;
    }

    public static int getTexHeight() {
        return texH;
    }

    private static void download(String fetchUrl) {
        try {
            byte[] bytes = fetchBytes(fetchUrl);
            if (bytes == null || bytes.length == 0) {
                loading.set(false);
                return;
            }
            registerFromBytesAsync(fetchUrl, bytes);
        } catch (Exception ignored) {
            loading.set(false);
        }
    }

    private static void registerFromBytesAsync(final String fetchUrl, byte[] bytes) {
        try {
            final BufferedImage image = ImageIO.read(new ByteArrayInputStream(bytes));
            if (image == null) {
                loading.set(false);
                return;
            }
            Minecraft.getMinecraft().addScheduledTask(new Runnable() {
                @Override
                public void run() {
                    if (!fetchUrl.equals(cachedUrl)) {
                        loading.set(false);
                        return;
                    }
                    try {
                        DynamicTexture dt = new DynamicTexture(image);
                        texW = Math.max(1, image.getWidth());
                        texH = Math.max(1, image.getHeight());
                        texture = Minecraft.getMinecraft().getTextureManager()
                            .getDynamicTextureLocation("music_art", dt);
                    } finally {
                        loading.set(false);
                    }
                }
            });
        } catch (Exception ignored) {
            loading.set(false);
        }
    }

    private static void registerFromBytes(String fetchUrl, byte[] bytes) {
        try {
            BufferedImage image = ImageIO.read(new ByteArrayInputStream(bytes));
            if (image == null || !fetchUrl.equals(cachedUrl)) {
                return;
            }
            DynamicTexture dt = new DynamicTexture(image);
            texW = Math.max(1, image.getWidth());
            texH = Math.max(1, image.getHeight());
            texture = Minecraft.getMinecraft().getTextureManager()
                .getDynamicTextureLocation("music_art", dt);
        } catch (Exception ignored) {
        }
    }

    private static byte[] fetchBytes(String fetchUrl) throws Exception {
        if (fetchUrl.startsWith("file://")) {
            return fetchLocalFile(fetchUrl);
        }
        if (fetchUrl.startsWith("http://") || fetchUrl.startsWith("https://")) {
            byte[] cached = readLauncherCache(fetchUrl);
            if (cached != null) {
                return cached;
            }
            return fetchHttp(fetchUrl);
        }
        return null;
    }

    private static byte[] fetchHttp(String fetchUrl) throws Exception {
        HttpURLConnection conn = (HttpURLConnection) new URL(fetchUrl).openConnection();
        conn.setInstanceFollowRedirects(true);
        conn.setConnectTimeout(5000);
        conn.setReadTimeout(8000);
        conn.setRequestProperty("User-Agent", "ParaguacraftPvP/2.0");
        conn.setRequestProperty("Accept", "image/avif,image/webp,image/apng,image/*,*/*;q=0.8");
        conn.connect();
        int code = conn.getResponseCode();
        if (code < 200 || code >= 300) {
            return null;
        }
        try (InputStream in = conn.getInputStream(); ByteArrayOutputStream out = new ByteArrayOutputStream()) {
            byte[] buf = new byte[8192];
            int read;
            while ((read = in.read(buf)) != -1) {
                out.write(buf, 0, read);
            }
            return out.toByteArray();
        }
    }

    private static byte[] readCachedBytes(String url) {
        if (url.startsWith("http://") || url.startsWith("https://")) {
            byte[] fromHash = readLauncherCache(url);
            if (fromHash != null) {
                return fromHash;
            }
            return readLatestLauncherCacheFile();
        }
        if (url.startsWith("file://")) {
            try {
                return fetchLocalFile(url);
            } catch (Exception ignored) {
            }
        }
        return null;
    }

    private static byte[] readLauncherCache(String httpsUrl) {
        Path path = launcherCachePath(httpsUrl);
        if (path == null || !Files.isRegularFile(path)) {
            return null;
        }
        try {
            return Files.readAllBytes(path);
        } catch (Exception e) {
            return null;
        }
    }

    private static byte[] readLatestLauncherCacheFile() {
        Path dir = launcherCacheDir();
        if (!Files.isDirectory(dir)) {
            return null;
        }
        try {
            Path newest = null;
            long newestMs = Long.MIN_VALUE;
            for (Path p : Files.newDirectoryStream(dir)) {
                String name = p.getFileName().toString().toLowerCase();
                if (!(name.endsWith(".jpg") || name.endsWith(".jpeg") || name.endsWith(".png"))) {
                    continue;
                }
                long ms = Files.getLastModifiedTime(p).toMillis();
                if (ms > newestMs) {
                    newestMs = ms;
                    newest = p;
                }
            }
            return newest != null ? Files.readAllBytes(newest) : null;
        } catch (Exception e) {
            return null;
        }
    }

    private static Path launcherCachePath(String httpsUrl) {
        String hash = sha1Hex(httpsUrl.getBytes(StandardCharsets.UTF_8));
        if (hash.isEmpty()) {
            return null;
        }
        return launcherCacheDir().resolve(hash + ".jpg");
    }

    private static String sha1Hex(byte[] bytes) {
        try {
            MessageDigest md = MessageDigest.getInstance("SHA-1");
            byte[] digest = md.digest(bytes);
            StringBuilder sb = new StringBuilder(digest.length * 2);
            for (byte b : digest) {
                sb.append(String.format("%02x", b & 0xff));
            }
            return sb.toString();
        } catch (Exception e) {
            return "";
        }
    }

    private static byte[] fetchLocalFile(String fileUrl) throws Exception {
        Path path = fileUrlToPath(fileUrl);
        if (Files.isRegularFile(path)) {
            return Files.readAllBytes(path);
        }
        Path name = path.getFileName();
        if (name != null) {
            Path byName = launcherCacheDir().resolve(name.toString());
            if (Files.isRegularFile(byName)) {
                return Files.readAllBytes(byName);
            }
        }
        return null;
    }

    private static Path fileUrlToPath(String fileUrl) {
        String raw = fileUrl;
        if (raw.regionMatches(true, 0, "file://", 0, 7)) {
            raw = raw.substring(7);
        }
        if (raw.startsWith("?/")) {
            raw = raw.substring(2);
        } else if (raw.startsWith("/?/")) {
            raw = raw.substring(3);
        }
        while (raw.startsWith("/") && raw.length() > 2 && raw.charAt(2) == ':') {
            raw = raw.substring(1);
        }
        try {
            return Paths.get(URI.create("file:///" + raw.replace('\\', '/')));
        } catch (Exception ignored) {
            return Paths.get(raw);
        }
    }

    private static Path launcherCacheDir() {
        String appData = System.getenv("APPDATA");
        return appData != null && !appData.isEmpty()
            ? Paths.get(appData, "ParaguacraftLauncher", "music-art")
            : Paths.get(System.getProperty("user.home"), ".config", "ParaguacraftLauncher", "music-art");
    }

    private static String normalizeUrl(String raw) {
        if (raw == null) {
            return "";
        }
        String trimmed = raw.trim();
        int https = trimmed.indexOf("https://");
        int http = trimmed.indexOf("http://");
        int file = trimmed.indexOf("file://");
        int start = https >= 0 ? https : (http >= 0 ? http : file);
        if (start < 0) {
            return "";
        }
        String url = trimmed.substring(start);
        int end = url.length();
        for (int i = 0; i < url.length(); i++) {
            char c = url.charAt(i);
            if (c <= 32) {
                end = i;
                break;
            }
        }
        return url.substring(0, end);
    }

    public static void clear() {
        cachedUrl = "";
        texture = null;
        texW = 64;
        texH = 64;
        loading.set(false);
    }
}
