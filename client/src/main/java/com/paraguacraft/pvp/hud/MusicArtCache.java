package com.paraguacraft.pvp.hud;

import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.texture.DynamicTexture;
import net.minecraft.util.ResourceLocation;

import javax.imageio.ImageIO;
import java.awt.image.BufferedImage;
import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.util.concurrent.atomic.AtomicBoolean;

/** Descarga y cachea la caratula de Spotify para el HUD de musica. */
public final class MusicArtCache {

    private static String cachedUrl = "";
    private static ResourceLocation texture;
    private static final AtomicBoolean loading = new AtomicBoolean(false);

    private MusicArtCache() {}

    public static ResourceLocation get(String url) {
        if (url == null || url.isEmpty()) {
            return null;
        }
        if (!url.equals(cachedUrl)) {
            cachedUrl = url;
            texture = null;
            loading.set(false);
        }
        if (texture != null) {
            return texture;
        }
        if (loading.compareAndSet(false, true)) {
            final String fetchUrl = url;
            Thread t = new Thread(new Runnable() {
                @Override
                public void run() {
                    try {
                        HttpURLConnection conn = (HttpURLConnection) new URL(fetchUrl).openConnection();
                        conn.setConnectTimeout(4000);
                        conn.setReadTimeout(6000);
                        conn.setRequestProperty("User-Agent", "ParaguacraftPvP/2.0");
                        conn.connect();
                        InputStream in = conn.getInputStream();
                        final BufferedImage image = ImageIO.read(in);
                        in.close();
                        if (image != null) {
                            Minecraft.getMinecraft().addScheduledTask(new Runnable() {
                                @Override
                                public void run() {
                                    DynamicTexture dt = new DynamicTexture(image);
                                    texture = Minecraft.getMinecraft().getTextureManager()
                                        .getDynamicTextureLocation("music_art", dt);
                                    loading.set(false);
                                }
                            });
                            return;
                        }
                    } catch (Exception ignored) {
                        /* retry next poll */
                    }
                    loading.set(false);
                }
            }, "Paraguacraft-MusicArt");
            t.setDaemon(true);
            t.start();
        }
        return null;
    }

    public static void clear() {
        cachedUrl = "";
        texture = null;
        loading.set(false);
    }
}
