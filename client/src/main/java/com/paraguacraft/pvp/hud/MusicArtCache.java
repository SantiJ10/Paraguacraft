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
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.concurrent.atomic.AtomicBoolean;

/** Descarga y cachea la caratula de Spotify/YouTube para el HUD de musica. */
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
        if (!url.equals(cachedUrl)) {
            cachedUrl = url;
            texture = null;
            texW = 64;
            texH = 64;
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
                    DynamicTexture dt = new DynamicTexture(image);
                    texW = image.getWidth();
                    texH = image.getHeight();
                    texture = Minecraft.getMinecraft().getTextureManager()
                        .getDynamicTextureLocation("music_art", dt);
                    loading.set(false);
                }
            });
        } catch (Exception ignored) {
            loading.set(false);
        }
    }

    private static byte[] fetchBytes(String fetchUrl) throws Exception {
        if (fetchUrl.startsWith("file://")) {
            Path path = fileUrlToPath(fetchUrl);
            if (!Files.isRegularFile(path)) {
                return null;
            }
            return Files.readAllBytes(path);
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
            byte[] buf = new byte[8192];
            int read;
            while ((read = in.read(buf)) != -1) {
                out.write(buf, 0, read);
            }
            return out.toByteArray();
        }
    }

    private static Path fileUrlToPath(String fileUrl) throws Exception {
        try {
            return Paths.get(URI.create(fileUrl));
        } catch (Exception ignored) {
            String raw = fileUrl.substring("file://".length());
            if (raw.startsWith("/") && raw.length() > 2 && raw.charAt(2) == ':') {
                raw = raw.substring(1);
            }
            return Paths.get(raw);
        }
    }

    public static void clear() {
        cachedUrl = "";
        texture = null;
        texW = 64;
        texH = 64;
        loading.set(false);
    }
}
