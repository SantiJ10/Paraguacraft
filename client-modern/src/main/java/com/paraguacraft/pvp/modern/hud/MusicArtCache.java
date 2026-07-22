package com.paraguacraft.pvp.modern.hud;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.texture.NativeImage;
import net.minecraft.client.texture.NativeImageBackedTexture;
import net.minecraft.util.Identifier;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

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

/** Descarga y cachea la caratula de Spotify/YouTube (portada original, sin regenerar). */
public final class MusicArtCache {

    private static final Logger LOGGER = LoggerFactory.getLogger("ParaguacraftPvP-MusicArt");
    public static final int DISPLAY_PX = 16;

    private static String cachedUrl = "";
    private static Identifier textureId;
    private static NativeImageBackedTexture texture;
    private static int texW = 1;
    private static int texH = 1;
    private static final AtomicBoolean loading = new AtomicBoolean(false);

    private MusicArtCache() {}

    public static Identifier get(String url) {
        if (url == null || url.isEmpty()) {
            return null;
        }
        String normalized = normalizeUrl(url);
        if (normalized.isEmpty()) {
            return null;
        }
        if (!normalized.equals(cachedUrl)) {
            cachedUrl = normalized;
            clearTexture();
            loading.set(false);
        }
        if (textureId != null) {
            return textureId;
        }
        byte[] cached = readCachedBytes(normalized);
        if (cached != null && cached.length > 0) {
            registerFromBytes(normalized, cached);
            return textureId;
        }
        if (loading.compareAndSet(false, true)) {
            final String fetchUrl = normalized;
            Thread t = new Thread(() -> download(fetchUrl), "Paraguacraft-MusicArt");
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
        } catch (Exception e) {
            LOGGER.warn("No se pudo descargar/decodificar la caratula ({})", fetchUrl, e);
            loading.set(false);
        }
    }

    private static void registerFromBytesAsync(String fetchUrl, byte[] bytes) {
        try {
            NativeImage image = decodeImage(bytes);
            if (image == null) {
                loading.set(false);
                return;
            }
            MinecraftClient client = MinecraftClient.getInstance();
            if (client == null) {
                image.close();
                loading.set(false);
                return;
            }
            client.execute(() -> {
                if (!fetchUrl.equals(cachedUrl)) {
                    image.close();
                    loading.set(false);
                    return;
                }
                try {
                    registerImage(image);
                } catch (Exception e) {
                    LOGGER.warn("No se pudo registrar la textura de la caratula ({})", fetchUrl, e);
                    image.close();
                } finally {
                    loading.set(false);
                }
            });
        } catch (Exception e) {
            LOGGER.warn("No se pudo decodificar la caratula ({})", fetchUrl, e);
            loading.set(false);
        }
    }

    private static void registerFromBytes(String fetchUrl, byte[] bytes) {
        try {
            NativeImage image = decodeImage(bytes);
            if (image == null) {
                return;
            }
            MinecraftClient client = MinecraftClient.getInstance();
            if (client == null || !fetchUrl.equals(cachedUrl)) {
                image.close();
                return;
            }
            registerImage(image);
        } catch (Exception e) {
            LOGGER.debug("Cache local de caratula no usable ({})", fetchUrl, e);
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
            in.transferTo(out);
            return out.toByteArray();
        }
    }

    private static byte[] readCachedBytes(String httpsUrl) {
        if (httpsUrl.startsWith("http://") || httpsUrl.startsWith("https://")) {
            byte[] fromHash = readLauncherCache(httpsUrl);
            if (fromHash != null) {
                return fromHash;
            }
            return readLatestLauncherCacheFile();
        }
        if (httpsUrl.startsWith("file://")) {
            try {
                Path path = fileUrlToPath(httpsUrl);
                if (Files.isRegularFile(path)) {
                    return Files.readAllBytes(path);
                }
            } catch (Exception ignored) {
            }
        }
        return null;
    }

    private static byte[] readLatestLauncherCacheFile() {
        Path dir = launcherCacheDir();
        if (!Files.isDirectory(dir)) {
            return null;
        }
        try (var stream = Files.list(dir)) {
            return stream
                .filter(p -> {
                    String name = p.getFileName().toString().toLowerCase();
                    return name.endsWith(".jpg") || name.endsWith(".jpeg") || name.endsWith(".png");
                })
                .max((a, b) -> {
                    try {
                        return Files.getLastModifiedTime(a).compareTo(Files.getLastModifiedTime(b));
                    } catch (Exception e) {
                        return 0;
                    }
                })
                .map(p -> {
                    try {
                        return Files.readAllBytes(p);
                    } catch (Exception e) {
                        return null;
                    }
                })
                .orElse(null);
        } catch (Exception e) {
            return null;
        }
    }

    private static byte[] readLauncherCache(String httpsUrl) {
        Path path = launcherCachePath(httpsUrl);
        if (path == null || !Files.isRegularFile(path)) {
            return null;
        }
        try {
            return Files.readAllBytes(path);
        } catch (Exception e) {
            LOGGER.debug("Cache local de caratula no legible ({})", path, e);
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
                sb.append(String.format("%02x", b));
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
        String name = path.getFileName() != null ? path.getFileName().toString() : "";
        if (!name.isEmpty()) {
            Path byName = launcherCacheDir().resolve(name);
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
        // Windows canonicalize a veces produce file://?/C:/... (prefijo \\?\ del SO).
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
            return Path.of(raw);
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

    private static NativeImage decodeImage(byte[] bytes) throws Exception {
        try (ByteArrayInputStream in = new ByteArrayInputStream(bytes)) {
            BufferedImage src = ImageIO.read(in);
            if (src != null) {
                return bufferedToNative(src);
            }
        }
        try (ByteArrayInputStream in = new ByteArrayInputStream(bytes)) {
            return NativeImage.read(in);
        }
    }

    private static NativeImage bufferedToNative(BufferedImage src) {
        int w = src.getWidth();
        int h = src.getHeight();
        NativeImage out = new NativeImage(w, h, false);
        for (int y = 0; y < h; y++) {
            for (int x = 0; x < w; x++) {
                out.setColorArgb(x, y, src.getRGB(x, y));
            }
        }
        return out;
    }

    private static void registerImage(NativeImage image) {
        clearTexture();
        texW = Math.max(1, image.getWidth());
        texH = Math.max(1, image.getHeight());
        texture = new NativeImageBackedTexture(() -> "paraguacraft_music_art", image);
        textureId = Identifier.of("paraguacraftpvp-modern", "music_art/live");
        MinecraftClient.getInstance().getTextureManager().registerTexture(textureId, texture);
    }

    private static void clearTexture() {
        if (textureId != null) {
            MinecraftClient client = MinecraftClient.getInstance();
            if (client != null) {
                client.getTextureManager().destroyTexture(textureId);
            }
        }
        textureId = null;
        texture = null;
        texW = 1;
        texH = 1;
    }

    public static void clear() {
        cachedUrl = "";
        clearTexture();
        loading.set(false);
    }
}
