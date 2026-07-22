package com.paraguacraft.pvp.modern.resourcepack;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.resource.ResourcePackManager;

import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.file.Files;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;

public final class ResourcePackService {

    public static final String OFFICIAL_PACK = "paraguacraft-pvp-modern.zip";

    public interface ProgressListener {
        void onProgress(String status, float ratio);
        void onComplete(String fileName);
        void onError(String message);
    }

    private ResourcePackService() {}

    public static File packsDir() {
        return new File(MinecraftClient.getInstance().runDirectory, "resourcepacks");
    }

    public static List<String> listInstalledZipNames() {
        List<String> out = new ArrayList<>();
        File dir = packsDir();
        if (!dir.isDirectory()) {
            return out;
        }
        File[] files = dir.listFiles();
        if (files == null) {
            return out;
        }
        for (File f : files) {
            if (f.isFile() && f.getName().toLowerCase().endsWith(".zip")) {
                out.add(f.getName());
            }
        }
        out.sort(String.CASE_INSENSITIVE_ORDER);
        return out;
    }

    public static boolean isInstalled(String fileName) {
        if (fileName == null || fileName.isBlank()) {
            return false;
        }
        return new File(packsDir(), fileName).isFile();
    }

    public static void download(CatalogPack pack, ProgressListener listener) {
        Thread t = new Thread(() -> {
            try {
                listener.onProgress("Descargando " + pack.title + "…", 0.1f);
                File dir = packsDir();
                if (!dir.exists()) {
                    dir.mkdirs();
                }
                File dest = new File(dir, pack.fileName);
                downloadHttp(pack.downloadUrl, dest);
                if (pack.sha1 != null && !pack.sha1.isEmpty() && !verifySha1(dest, pack.sha1)) {
                    throw new IllegalStateException("SHA1 inválido");
                }
                listener.onComplete(pack.fileName);
            } catch (Exception e) {
                listener.onError(e.getMessage() != null ? e.getMessage() : "Error de descarga");
            }
        }, "Paraguacraft-PackDownload");
        t.setDaemon(true);
        t.start();
    }

    public static void applyPack(String fileName) {
        applyPacks(List.of(fileName));
    }

    /** Pack oficial + secundario opcional (orden: oficial primero). */
    public static void applySessionPacks() {
        List<String> packs = new ArrayList<>();
        String primary = ModernConfig.selectedResourcePack;
        if (primary == null || primary.isBlank()) {
            primary = OFFICIAL_PACK;
        }
        if (isInstalled(primary)) {
            packs.add(primary);
        } else if (isInstalled(OFFICIAL_PACK)) {
            packs.add(OFFICIAL_PACK);
        }
        String secondary = ModernConfig.secondaryResourcePack;
        if (secondary != null && !secondary.isBlank() && isInstalled(secondary) && !packs.contains(secondary)) {
            packs.add(secondary);
        }
        if (packs.isEmpty()) {
            return;
        }
        applyPacks(packs);
    }

    public static void applyPacks(List<String> fileNames) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client == null || fileNames == null || fileNames.isEmpty()) {
            return;
        }
        ResourcePackManager manager = client.getResourcePackManager();
        manager.scanPacks();
        Set<String> enabled = new LinkedHashSet<>();
        for (String fileName : fileNames) {
            if (!isInstalled(fileName)) {
                continue;
            }
            String packId = resolvePackId(manager, fileName);
            if (packId != null) {
                enabled.add(packId);
            }
        }
        if (enabled.isEmpty()) {
            return;
        }
        for (String id : manager.getEnabledIds()) {
            if (!id.startsWith("file/")) {
                enabled.add(id);
            }
        }
        manager.setEnabledProfiles(enabled);
        client.options.refreshResourcePacks(manager);
        client.options.write();
        if (!fileNames.isEmpty()) {
            ModernConfig.selectedResourcePack = fileNames.get(0);
        }
        ModernConfig.save();
        client.reloadResources();
    }

    public static void restoreSavedPack() {
        applySessionPacks();
    }

    public static void setSecondaryPack(String fileName) {
        ModernConfig.secondaryResourcePack = fileName == null ? "" : fileName;
        ModernConfig.save();
        applySessionPacks();
    }

    private static String resolvePackId(ResourcePackManager manager, String fileName) {
        String direct = "file/" + fileName;
        if (manager.getIds().contains(direct)) {
            return direct;
        }
        for (String id : manager.getIds()) {
            if (id.endsWith("/" + fileName) || id.endsWith(fileName)) {
                return id;
            }
        }
        return direct;
    }

    private static void downloadHttp(String urlStr, File dest) throws Exception {
        HttpURLConnection conn = (HttpURLConnection) new URL(urlStr).openConnection();
        conn.setRequestProperty("User-Agent", "Paraguacraft-Modern/1.0");
        conn.setConnectTimeout(12000);
        conn.setReadTimeout(60000);
        try (InputStream in = conn.getInputStream(); FileOutputStream out = new FileOutputStream(dest)) {
            byte[] buf = new byte[8192];
            int read;
            while ((read = in.read(buf)) >= 0) {
                out.write(buf, 0, read);
            }
        }
    }

    private static boolean verifySha1(File file, String expected) throws Exception {
        MessageDigest md = MessageDigest.getInstance("SHA-1");
        byte[] bytes = Files.readAllBytes(file.toPath());
        byte[] digest = md.digest(bytes);
        StringBuilder sb = new StringBuilder();
        for (byte b : digest) {
            sb.append(String.format("%02x", b));
        }
        return sb.toString().equalsIgnoreCase(expected.trim());
    }
}
