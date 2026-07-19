package com.paraguacraft.pvp.modern.hud;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.texture.NativeImage;
import net.minecraft.client.texture.NativeImageBackedTexture;
import net.minecraft.util.Identifier;

import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URI;
import java.util.concurrent.atomic.AtomicBoolean;

/** Descarga y cachea la caratula de Spotify/YouTube para el HUD de musica. */
public final class MusicArtCache {

    private static String cachedUrl = "";
    private static Identifier textureId;
    private static NativeImageBackedTexture texture;
    private static int texW = 64;
    private static int texH = 64;
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
            HttpURLConnection conn = (HttpURLConnection) URI.create(fetchUrl).toURL().openConnection();
            conn.setInstanceFollowRedirects(true);
            conn.setConnectTimeout(4000);
            conn.setReadTimeout(8000);
            conn.setRequestProperty("User-Agent", "ParaguacraftPvP-Modern/0.6");
            conn.setRequestProperty("Accept", "image/*,*/*");
            conn.connect();
            try (InputStream in = conn.getInputStream()) {
                NativeImage image = NativeImage.read(in);
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
            }
        } catch (Exception ignored) {
            /* retry next poll */
        }
        loading.set(false);
    }

    private static void registerImage(NativeImage image, int w, int h) {
        clearTexture();
        texW = Math.max(1, w);
        texH = Math.max(1, h);
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
        texW = 64;
        texH = 64;
    }

    public static void clear() {
        cachedUrl = "";
        clearTexture();
        loading.set(false);
    }
}
