package com.paraguacraft.pvp.modern.hud;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.texture.NativeImage;
import net.minecraft.client.texture.NativeImageBackedTexture;
import net.minecraft.util.Identifier;

import javax.imageio.ImageIO;
import java.awt.Graphics2D;
import java.awt.RenderingHints;
import java.awt.image.BufferedImage;
import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * Descarga la portada real de Spotify/YouTube (mismo enfoque que 1.8.9: ImageIO + DynamicTexture).
 */
public final class MusicArtCache {

    public static final int DISPLAY_PX = 16;
    private static final int TEX_PX = 64;
    private static final long RETRY_MS = 8_000L;

    private static String cachedUrl = "";
    private static String failedUrl = "";
    private static long failedAt;
    private static Identifier textureId;
    private static NativeImageBackedTexture texture;
    private static int texW = TEX_PX;
    private static int texH = TEX_PX;
    private static final AtomicBoolean loading = new AtomicBoolean(false);

    private MusicArtCache() {}

    /** Inicia descarga si hace falta; devuelve textura lista o null mientras carga/falla. */
    public static Identifier get(String url) {
        if (url == null || url.isEmpty()) {
            return null;
        }
        if (!url.equals(cachedUrl)) {
            cachedUrl = url;
            failedUrl = "";
            clearTexture();
            loading.set(false);
        }
        if (textureId != null) {
            return textureId;
        }
        if (url.equals(failedUrl) && System.currentTimeMillis() - failedAt < RETRY_MS) {
            return null;
        }
        if (loading.compareAndSet(false, true)) {
            final String fetchUrl = url;
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
                markFailed(fetchUrl);
                return;
            }
            NativeImage image = decodeImage(bytes);
            if (image == null) {
                markFailed(fetchUrl);
                return;
            }
            MinecraftClient client = MinecraftClient.getInstance();
            if (client == null) {
                image.close();
                markFailed(fetchUrl);
                return;
            }
            client.execute(() -> registerImage(image));
        } catch (Exception ignored) {
            markFailed(fetchUrl);
        }
    }

    private static byte[] fetchBytes(String fetchUrl) throws Exception {
        HttpURLConnection conn = (HttpURLConnection) new URL(fetchUrl).openConnection();
        conn.setInstanceFollowRedirects(true);
        conn.setConnectTimeout(8000);
        conn.setReadTimeout(15000);
        conn.setRequestProperty("User-Agent", "ParaguacraftPvP/2.0");
        conn.setRequestProperty("Accept", "image/avif,image/webp,image/apng,image/*,*/*;q=0.8");
        if (fetchUrl.contains("scdn.co") || fetchUrl.contains("spotify")) {
            conn.setRequestProperty("Referer", "https://open.spotify.com/");
        }
        if (fetchUrl.contains("ytimg.com") || fetchUrl.contains("youtube.com")) {
            conn.setRequestProperty("Referer", "https://www.youtube.com/");
        }
        conn.connect();
        try (InputStream in = conn.getInputStream(); ByteArrayOutputStream out = new ByteArrayOutputStream()) {
            in.transferTo(out);
            return out.toByteArray();
        }
    }

    /** ImageIO primero (1.8.9); NativeImage.read como respaldo. */
    private static NativeImage decodeImage(byte[] bytes) throws Exception {
        BufferedImage src = ImageIO.read(new ByteArrayInputStream(bytes));
        if (src != null) {
            return scaleToTexture(fromBufferedImage(src));
        }
        NativeImage direct = NativeImage.read(new ByteArrayInputStream(bytes));
        if (direct == null) {
            return null;
        }
        return scaleToTexture(direct);
    }

    private static NativeImage fromBufferedImage(BufferedImage src) {
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

    private static NativeImage scaleToTexture(NativeImage src) {
        int w = src.getWidth();
        int h = src.getHeight();
        if (w <= 0 || h <= 0) {
            src.close();
            return null;
        }
        if (w == TEX_PX && h == TEX_PX) {
            return src;
        }
        NativeImage out = new NativeImage(TEX_PX, TEX_PX, false);
        for (int y = 0; y < TEX_PX; y++) {
            for (int x = 0; x < TEX_PX; x++) {
                int sx = x * w / TEX_PX;
                int sy = y * h / TEX_PX;
                out.setColorArgb(x, y, src.getColorArgb(sx, sy));
            }
        }
        src.close();
        return out;
    }

    private static void markFailed(String url) {
        failedUrl = url;
        failedAt = System.currentTimeMillis();
        loading.set(false);
    }

    private static void registerImage(NativeImage image) {
        clearTexture();
        texW = Math.max(1, image.getWidth());
        texH = Math.max(1, image.getHeight());
        texture = new NativeImageBackedTexture(() -> "paraguacraft_music_art", image);
        textureId = Identifier.of("paraguacraftpvp-modern", "music_art/live");
        MinecraftClient.getInstance().getTextureManager().registerTexture(textureId, texture);
        loading.set(false);
        failedUrl = "";
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
        texW = TEX_PX;
        texH = TEX_PX;
    }

    public static void clear() {
        cachedUrl = "";
        failedUrl = "";
        clearTexture();
        loading.set(false);
    }
}
