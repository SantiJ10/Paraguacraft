package com.paraguacraft.pvp.modern.hud;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.texture.NativeImage;
import net.minecraft.client.texture.NativeImageBackedTexture;
import net.minecraft.util.Identifier;

import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * Descarga y cachea la portada real de Spotify/YouTube (mismo flujo que 1.8.9 DynamicTexture).
 * Usa {@link NativeImage#read(InputStream)} sin conversion manual de pixeles.
 */
public final class MusicArtCache {

    /** Dibujado en HUD a 16px; la textura conserva la resolucion nativa de la portada. */
    public static final int DISPLAY_PX = 16;

    private static final long RETRY_MS = 10_000L;

    private static String cachedUrl = "";
    private static String failedUrl = "";
    private static long failedAt;
    private static Identifier textureId;
    private static NativeImageBackedTexture texture;
    private static int texW = DISPLAY_PX;
    private static int texH = DISPLAY_PX;
    private static final AtomicBoolean loading = new AtomicBoolean(false);

    private MusicArtCache() {}

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
            HttpURLConnection conn = (HttpURLConnection) new URL(fetchUrl).openConnection();
            conn.setInstanceFollowRedirects(true);
            conn.setConnectTimeout(8000);
            conn.setReadTimeout(15000);
            conn.setRequestProperty("User-Agent", "ParaguacraftPvP-Modern/1.0");
            conn.setRequestProperty("Accept", "image/jpeg,image/png,image/webp,image/*,*/*");
            if (fetchUrl.contains("scdn.co") || fetchUrl.contains("spotify.com")) {
                conn.setRequestProperty("Referer", "https://open.spotify.com/");
            }
            if (fetchUrl.contains("ytimg.com") || fetchUrl.contains("youtube.com")) {
                conn.setRequestProperty("Referer", "https://www.youtube.com/");
            }
            int code = conn.getResponseCode();
            if (code < 200 || code >= 300) {
                markFailed(fetchUrl);
                return;
            }
            try (InputStream in = conn.getInputStream()) {
                NativeImage image = NativeImage.read(in);
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
                return;
            }
        } catch (Exception ignored) {
            /* reintento mas tarde */
        }
        markFailed(fetchUrl);
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
        texW = DISPLAY_PX;
        texH = DISPLAY_PX;
    }

    public static void clear() {
        cachedUrl = "";
        failedUrl = "";
        clearTexture();
        loading.set(false);
    }
}
