package com.paraguacraft.pvp.modern.core;

import com.mojang.authlib.GameProfile;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.texture.NativeImage;
import net.minecraft.client.texture.NativeImageBackedTexture;
import net.minecraft.util.Identifier;

import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.file.Files;
import java.nio.file.Path;

/** Aplica skins custom por URL o nick (estilo Lunar). */
public final class SkinManager {

    private static final Identifier CUSTOM_TEX = Identifier.of("paraguacraftpvp-modern", "custom_skin");
    private static volatile String activeUrl = "";
    private static volatile boolean hasCustom;

    private SkinManager() {}

    public static boolean hasCustomSkin() {
        return hasCustom;
    }

    public static Identifier customTexture() {
        return CUSTOM_TEX;
    }

    public static void apply(String value) {
        if (value == null || value.isBlank()) {
            clear();
            return;
        }
        activeUrl = value.trim();
        Thread t = new Thread(() -> {
            try {
                String url = resolveUrl(activeUrl);
                if (url == null) {
                    return;
                }
                downloadAndRegister(url);
                hasCustom = true;
            } catch (Exception ignored) {
            }
        }, "Paraguacraft-Skin");
        t.setDaemon(true);
        t.start();
    }

    public static void clear() {
        activeUrl = "";
        hasCustom = false;
        MinecraftClient client = MinecraftClient.getInstance();
        if (client != null) {
            client.execute(() -> client.getTextureManager().destroyTexture(CUSTOM_TEX));
        }
    }

    private static String resolveUrl(String value) {
        if (value.startsWith("http://") || value.startsWith("https://") || value.startsWith("file://")) {
            return value;
        }
        return "https://minotar.net/skin/" + value;
    }

    /** Skins unificadas (Fase 2.3): `file://` = misma skin offline que aplicó el launcher (sin red). */
    private static void downloadAndRegister(String urlStr) throws Exception {
        NativeImage image;
        if (urlStr.startsWith("file://")) {
            Path path = Path.of(urlStr.substring("file://".length()));
            if (!Files.isRegularFile(path)) {
                return;
            }
            try (InputStream in = Files.newInputStream(path)) {
                image = NativeImage.read(in);
            }
        } else {
            HttpURLConnection conn = (HttpURLConnection) new URL(urlStr).openConnection();
            conn.setRequestProperty("User-Agent", "Paraguacraft-Modern/1.0");
            conn.setConnectTimeout(8000);
            conn.setReadTimeout(15000);
            try (InputStream in = conn.getInputStream()) {
                image = NativeImage.read(in);
            }
        }
        NativeImage finalImage = image;
        MinecraftClient.getInstance().execute(() -> {
            NativeImageBackedTexture tex = new NativeImageBackedTexture(() -> "paraguacraft_custom_skin", finalImage);
            MinecraftClient.getInstance().getTextureManager().registerTexture(CUSTOM_TEX, tex);
        });
    }

    /** Reservado para mixin de skin; la textura custom se registra en {@link #apply}. */
    public static GameProfile overlayProfile(GameProfile base) {
        return base;
    }
}
