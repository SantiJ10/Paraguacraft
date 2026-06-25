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
        gs.entityShadows = false;
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
        patches.put("ofAnimatedWater", "2");
        patches.put("ofAnimatedLava", "2");
        patches.put("ofAnimatedFire", "2");
        patches.put("ofAnimatedPortal", "2");
        patches.put("ofAnimatedRedstone", "2");
        patches.put("ofAnimatedExplosion", "2");
        patches.put("ofAnimatedFlame", "2");
        patches.put("ofAnimatedSmoke", "2");
        patches.put("ofLazyChunkLoading", "true");
        patches.put("ofChunkUpdates", "1");
        patches.put("ofFastMath", "true");

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
