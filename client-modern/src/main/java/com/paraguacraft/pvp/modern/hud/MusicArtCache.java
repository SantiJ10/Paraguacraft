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
import java.net.URI;
import java.util.concurrent.atomic.AtomicBoolean;

/** Descarga y cachea la caratula de Spotify/YouTube para el HUD de musica. */
public final class MusicArtCache {

    /** Textura 64x64 como 1.8.9; se dibuja a 16px en pantalla. */
    private static final int TEX_PX = 64;
    private static final int DISPLAY_PX = 16;
    private static final long RETRY_MS = 12_000L;

    private static String cachedUrl = "";
    private static String failedUrl = "";
    private static long failedAt;
    private static Identifier textureId;
    private static NativeImageBackedTexture texture;
    private static int texW = TEX_PX;
    private static int texH = TEX_PX;
    private static final AtomicBoolean loading = new AtomicBoolean(false);

    private MusicArtCache() {}

    public static int displaySize() {
        return DISPLAY_PX;
    }

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
            HttpURLConnection conn = (HttpURLConnection) URI.create(fetchUrl).toURL().openConnection();
            conn.setInstanceFollowRedirects(true);
            conn.setConnectTimeout(5000);
            conn.setReadTimeout(10000);
            conn.setRequestProperty("User-Agent", "ParaguacraftPvP-Modern/0.6");
            conn.setRequestProperty("Accept", "image/jpeg,image/png,image/webp,image/*,*/*");
            if (fetchUrl.contains("scdn.co") || fetchUrl.contains("spotify")) {
                conn.setRequestProperty("Referer", "https://open.spotify.com/");
            }
            conn.connect();
            byte[] bytes;
            try (InputStream in = conn.getInputStream(); ByteArrayOutputStream out = new ByteArrayOutputStream()) {
                in.transferTo(out);
                bytes = out.toByteArray();
            }
            if (bytes.length == 0) {
                markFailed(fetchUrl);
                return;
            }
            NativeImage image = decodeImage(bytes);
            if (image != null) {
                final int w = image.getWidth();
                final int h = image.getHeight();
                MinecraftClient client = MinecraftClient.getInstance();
                if (client != null) {
                    client.execute(() -> registerImage(image, w, h));
                    return;
                }
                image.close();
            }
        } catch (Exception ignored) {
            /* retry later */
        }
        markFailed(fetchUrl);
    }

    private static void markFailed(String url) {
        failedUrl = url;
        failedAt = System.currentTimeMillis();
        loading.set(false);
    }

    private static NativeImage decodeImage(byte[] bytes) throws Exception {
        BufferedImage src = ImageIO.read(new ByteArrayInputStream(bytes));
        if (src != null) {
            BufferedImage scaled = new BufferedImage(TEX_PX, TEX_PX, BufferedImage.TYPE_INT_ARGB);
            Graphics2D g = scaled.createGraphics();
            g.setRenderingHint(RenderingHints.KEY_INTERPOLATION, RenderingHints.VALUE_INTERPOLATION_BILINEAR);
            g.drawImage(src, 0, 0, TEX_PX, TEX_PX, null);
            g.dispose();
            NativeImage out = new NativeImage(TEX_PX, TEX_PX, false);
            for (int y = 0; y < TEX_PX; y++) {
                for (int x = 0; x < TEX_PX; x++) {
                    out.setColorArgb(x, y, scaled.getRGB(x, y));
                }
            }
            return out;
        }
        NativeImage direct = NativeImage.read(new ByteArrayInputStream(bytes));
        if (direct == null) {
            return null;
        }
        return resizeBilinear(direct);
    }

    private static NativeImage resizeBilinear(NativeImage src) {
        int w = src.getWidth();
        int h = src.getHeight();
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

    private static void registerImage(NativeImage image, int w, int h) {
        clearTexture();
        texW = Math.max(1, w);
        texH = Math.max(1, h);
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
