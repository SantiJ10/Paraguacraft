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
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.concurrent.atomic.AtomicBoolean;

/** Descarga y cachea la caratula de Spotify/YouTube (mismo flujo que 1.8.9). */
public final class MusicArtCache {

    public static final int DISPLAY_PX = 16;
    private static final int TEX_PX = 64;

    private static String cachedUrl = "";
    private static Identifier textureId;
    private static NativeImageBackedTexture texture;
    private static int texW = TEX_PX;
    private static int texH = TEX_PX;
    private static final AtomicBoolean loading = new AtomicBoolean(false);

    private MusicArtCache() {}

    public static Identifier get(String url) {
        if (url == null || url.isEmpty()) {
            return null;
        }
        if (!url.equals(cachedUrl)) {
            cachedUrl = url;
            clearTexture();
            loading.set(false);
        }
        if (textureId != null) {
            return textureId;
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
                loading.set(false);
                return;
            }
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
                registerImage(image);
            });
        } catch (Exception ignored) {
            loading.set(false);
        }
    }

    /**
     * Prioridad: archivo local ya cacheado por el launcher (`file://...`,
     * ver `music_art_cache.rs`) → HTTP directo. El local evita red por
     * completo (mas rapido y sin depender de que ImageIO soporte el Content-Type).
     */
    private static byte[] fetchBytes(String fetchUrl) throws Exception {
        if (fetchUrl.startsWith("file://")) {
            return fetchLocalFile(fetchUrl);
        }
        HttpURLConnection conn = (HttpURLConnection) new URL(fetchUrl).openConnection();
        conn.setInstanceFollowRedirects(true);
        conn.setConnectTimeout(4000);
        conn.setReadTimeout(6000);
        conn.setRequestProperty("User-Agent", "ParaguacraftPvP/2.0");
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

    private static byte[] fetchLocalFile(String fileUrl) throws Exception {
        String raw = fileUrl.substring("file://".length());
        Path path = Path.of(raw);
        if (!Files.isRegularFile(path)) {
            return null;
        }
        return Files.readAllBytes(path);
    }

    private static NativeImage decodeImage(byte[] bytes) throws Exception {
        BufferedImage src = ImageIO.read(new ByteArrayInputStream(bytes));
        if (src != null) {
            return toNativeTexture(src);
        }
        NativeImage direct = NativeImage.read(new ByteArrayInputStream(bytes));
        if (direct == null) {
            return null;
        }
        BufferedImage fromNative = toBufferedImage(direct);
        direct.close();
        return toNativeTexture(fromNative);
    }

    private static NativeImage toNativeTexture(BufferedImage src) {
        BufferedImage scaled = src;
        if (src.getWidth() != TEX_PX || src.getHeight() != TEX_PX) {
            scaled = new BufferedImage(TEX_PX, TEX_PX, BufferedImage.TYPE_INT_ARGB);
            Graphics2D g = scaled.createGraphics();
            g.setRenderingHint(RenderingHints.KEY_INTERPOLATION, RenderingHints.VALUE_INTERPOLATION_BILINEAR);
            g.setRenderingHint(RenderingHints.KEY_RENDERING, RenderingHints.VALUE_RENDER_QUALITY);
            g.drawImage(src, 0, 0, TEX_PX, TEX_PX, null);
            g.dispose();
        }
        NativeImage out = new NativeImage(TEX_PX, TEX_PX, false);
        for (int y = 0; y < TEX_PX; y++) {
            for (int x = 0; x < TEX_PX; x++) {
                out.setColorArgb(x, y, scaled.getRGB(x, y));
            }
        }
        return out;
    }

    private static BufferedImage toBufferedImage(NativeImage src) {
        int w = src.getWidth();
        int h = src.getHeight();
        BufferedImage out = new BufferedImage(w, h, BufferedImage.TYPE_INT_ARGB);
        for (int y = 0; y < h; y++) {
            for (int x = 0; x < w; x++) {
                out.setRGB(x, y, src.getColorArgb(x, y));
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
        loading.set(false);
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
        clearTexture();
        loading.set(false);
    }
}
