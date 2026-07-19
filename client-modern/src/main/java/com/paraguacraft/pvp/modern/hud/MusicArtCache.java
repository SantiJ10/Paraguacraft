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
        return textureId;
    }

    public static void request(String url) {
        if (url == null || url.isEmpty() || url.equals(cachedUrl) && textureId != null) {
            return;
        }
        if (!url.equals(cachedUrl)) {
            cachedUrl = url;
            clearTexture();
            loading.set(false);
        }
        if (textureId != null || !loading.compareAndSet(false, true)) {
            return;
        }
        final String fetchUrl = url;
        Thread t = new Thread(() -> {
            try {
                HttpURLConnection conn = (HttpURLConnection) URI.create(fetchUrl).toURL().openConnection();
                conn.setConnectTimeout(4000);
                conn.setReadTimeout(6000);
                conn.setRequestProperty("User-Agent", "ParaguacraftPvP-Modern/0.6");
                conn.connect();
                try (InputStream in = conn.getInputStream()) {
                    NativeImage image = NativeImage.read(in);
                    if (image != null) {
                        MinecraftClient.getInstance().execute(() -> {
                            clearTexture();
                            texture = new NativeImageBackedTexture(() -> "paraguacraft_music_art", image);
                            textureId = Identifier.of("paraguacraftpvp-modern", "music_art/live");
                            MinecraftClient.getInstance().getTextureManager().registerTexture(textureId, texture);
                            loading.set(false);
                        });
                        return;
                    }
                }
            } catch (Exception ignored) {
                /* retry next poll */
            }
            loading.set(false);
        }, "Paraguacraft-MusicArt");
        t.setDaemon(true);
        t.start();
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
    }

    public static void clear() {
        cachedUrl = "";
        clearTexture();
        loading.set(false);
    }
}
