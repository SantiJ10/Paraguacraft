package com.paraguacraft.pvp.core;

import net.minecraft.client.Minecraft;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.file.Files;

/**
 * Pre-configura OneConfig + Hytils en la primera ejecución del cliente PvP:
 * - Quita Right Shift de OneConfig (Paraguacraft usa RShift para el Mod Menu).
 * - Hytils: solo camas coloreadas; el resto desactivado para no solaparse con Paraguacraft.
 */
public final class HytilsDefaults {
    private static final String MARKER = ".paraguacraft-hytils-v1";
    private static final String RESOURCE_PREFIX = "/defaults/oneconfig/";

    private HytilsDefaults() {}

    public static void applyIfNeeded() {
        File gameDir = Minecraft.getMinecraft().mcDataDir;
        File oneConfigDir = new File(gameDir, "OneConfig");
        File marker = new File(oneConfigDir, MARKER);
        if (marker.isFile()) {
            return;
        }

        try {
            oneConfigDir.mkdirs();
            copyResource("OneConfig.json", new File(oneConfigDir, "OneConfig.json"), true);
            copyResource("Preferences.json", new File(oneConfigDir, "Preferences.json"), false);
            copyResource(
                "profiles/Default Profile/hytilsreborn.json",
                new File(oneConfigDir, "profiles/Default Profile/hytilsreborn.json"),
                false
            );
            if (!marker.createNewFile()) {
                marker.setLastModified(System.currentTimeMillis());
            }
            System.out.println("[Paraguacraft] OneConfig/Hytils preconfigurado (camas + sin RShift en OneConfig).");
        } catch (IOException e) {
            System.err.println("[Paraguacraft] No se pudo preconfigurar Hytils/OneConfig: " + e.getMessage());
        }
    }

    private static void copyResource(String resourcePath, File dest, boolean onlyIfMissing) throws IOException {
        if (onlyIfMissing && dest.isFile()) {
            return;
        }
        File parent = dest.getParentFile();
        if (parent != null) {
            parent.mkdirs();
        }
        String fullPath = RESOURCE_PREFIX + resourcePath;
        InputStream in = HytilsDefaults.class.getResourceAsStream(fullPath);
        if (in == null) {
            throw new IOException("Recurso no encontrado: " + fullPath);
        }
        try (InputStream stream = in; OutputStream out = Files.newOutputStream(dest.toPath())) {
            byte[] buf = new byte[8192];
            int n;
            while ((n = stream.read(buf)) != -1) {
                out.write(buf, 0, n);
            }
        }
    }
}
