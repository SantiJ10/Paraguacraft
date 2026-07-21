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
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
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
            LOGGER.warn("No se pudo descargar/decodificar la caratula ({})", fetchUrl, e);
            loading.set(false);
        }
    }

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
        Path path = fileUrlToPath(fileUrl);
        if (!Files.isRegularFile(path)) {
            return null;
        }
        return Files.readAllBytes(path);
    }

    private static Path fileUrlToPath(String fileUrl) throws Exception {
        try {
            return Paths.get(URI.create(fileUrl));
        } catch (Exception ignored) {
            String raw = fileUrl.substring("file://".length());
            if (raw.startsWith("/") && raw.length() > 2 && raw.charAt(2) == ':') {
                raw = raw.substring(1);
            }
            return Path.of(raw);
        }
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
