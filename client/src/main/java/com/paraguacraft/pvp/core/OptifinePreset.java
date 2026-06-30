package com.paraguacraft.pvp.core;

import net.minecraft.client.Minecraft;
import net.minecraft.client.settings.GameSettings;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * Aplica preset de video vanilla + OptiFine cuando Boost FPS esta activo.
 */
public final class OptifinePreset {

    private OptifinePreset() {}

    public static void applyIfEnabled() {
        if (!PerformanceConfig.boostFps || !PerformanceConfig.applyVanillaPreset) {
            return;
        }
        applyParticlesOnly();
        applyVanillaGraphics();
        patchOptionsOf();
    }

    public static void applyParticlesOnly() {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc == null || mc.gameSettings == null) {
            return;
        }
        if (!PerformanceConfig.applyVanillaPreset) {
            return;
        }
        GameSettings gs = mc.gameSettings;
        gs.particleSetting = PerformanceConfig.particleMode.getVanillaParticles();
        gs.saveOptions();
    }

    private static void applyVanillaGraphics() {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc == null || mc.gameSettings == null) {
            return;
        }
        GameSettings gs = mc.gameSettings;
        // Estos sí aplican en vivo (próximo frame).
        gs.entityShadows = false;
        gs.fancyGraphics = false;   // Fast Graphics: gran ganancia de FPS en PvP.
        gs.ambientOcclusion = 0;    // Smooth Lighting OFF.
        gs.mipmapLevels = 0;        // Sin mipmaps: menos costo de muestreo.
        gs.saveOptions();
    }

    /** Parchea optionsof.txt (OptiFine 1.8.9) sin depender de clases OF en runtime. */
    private static void patchOptionsOf() {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc == null || mc.mcDataDir == null) {
            return;
        }
        File ofFile = new File(mc.mcDataDir, "optionsof.txt");
        Map<String, String> patches = new LinkedHashMap<String, String>();
        // Animaciones de texturas apagadas (2 = off en OptiFine).
        patches.put("ofAnimatedWater", "2");
        patches.put("ofAnimatedLava", "2");
        patches.put("ofAnimatedFire", "2");
        patches.put("ofAnimatedPortal", "2");
        patches.put("ofAnimatedRedstone", "2");
        patches.put("ofAnimatedExplosion", "2");
        patches.put("ofAnimatedFlame", "2");
        patches.put("ofAnimatedSmoke", "2");
        patches.put("ofWaterParticles", "false");
        patches.put("ofRainSplash", "false");
        patches.put("ofVoidParticles", "false");
        patches.put("ofPortalParticles", "false");
        patches.put("ofFireworkParticles", "false");
        // Carga/actualización de chunks.
        patches.put("ofLazyChunkLoading", "true");
        patches.put("ofChunkUpdates", "1");
        patches.put("ofFastMath", "true");
        // Lo que más FPS da en 1.8.9.
        patches.put("ofFastRender", "true");
        patches.put("ofRenderRegions", "true");
        patches.put("ofSmartAnimations", "true");
        patches.put("ofSmoothFps", "false");
        // Efectos pesados apagados (3 = off en OptiFine).
        patches.put("ofClouds", "3");
        patches.put("ofRain", "3");
        patches.put("ofDroppedItems", "0");
        patches.put("ofWeather", "false");
        patches.put("ofSky", "false");
        patches.put("ofStars", "false");
        patches.put("ofVignette", "1");
        patches.put("ofDynamicFov", "false");
        patches.put("ofDynamicLights", "3");
        // Texturas/calidad: lo mínimo para máximo FPS.
        patches.put("ofMipmapType", "0");
        patches.put("ofAaLevel", "0");
        patches.put("ofAfLevel", "1");
        patches.put("ofBetterGrass", "3");
        patches.put("ofConnectedTextures", "2");
        patches.put("ofCustomSky", "false");
        patches.put("ofCustomFonts", "false");
        patches.put("ofCustomColors", "false");
        patches.put("ofNaturalTextures", "false");
        patches.put("ofEmissiveTextures", "false");

        try {
            List<String> lines = new ArrayList<String>();
            if (ofFile.isFile()) {
                byte[] raw = readAllBytes(ofFile);
                String text = new String(raw, StandardCharsets.UTF_8);
                for (String line : text.split("\r?\n")) {
                    lines.add(line);
                }
            }
            for (Map.Entry<String, String> entry : patches.entrySet()) {
                replaceOrAdd(lines, entry.getKey(), entry.getValue());
            }
            StringBuilder out = new StringBuilder();
            for (int i = 0; i < lines.size(); i++) {
                if (i > 0) {
                    out.append('\n');
                }
                out.append(lines.get(i));
            }
            FileOutputStream fos = new FileOutputStream(ofFile);
            fos.write(out.toString().getBytes(StandardCharsets.UTF_8));
            fos.close();
        } catch (Exception ignored) {
        }
    }

    private static void replaceOrAdd(List<String> lines, String key, String value) {
        String prefix = key + ":";
        for (int i = 0; i < lines.size(); i++) {
            if (lines.get(i).startsWith(prefix)) {
                lines.set(i, prefix + value);
                return;
            }
        }
        lines.add(prefix + value);
    }

    private static byte[] readAllBytes(File file) throws java.io.IOException {
        FileInputStream in = new FileInputStream(file);
        byte[] data = new byte[(int) file.length()];
        int read = in.read(data);
        in.close();
        if (read < data.length) {
            byte[] trimmed = new byte[read];
            System.arraycopy(data, 0, trimmed, 0, read);
            return trimmed;
        }
        return data;
    }
}
