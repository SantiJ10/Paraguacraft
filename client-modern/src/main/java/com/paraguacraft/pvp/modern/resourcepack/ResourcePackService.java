package com.paraguacraft.pvp.modern.resourcepack;

import net.minecraft.client.MinecraftClient;

import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.file.Files;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.List;

public final class ResourcePackService {

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
        MinecraftClient client = MinecraftClient.getInstance();
        File file = new File(packsDir(), fileName);
        if (!file.isFile()) {
            return;
        }
        client.getResourcePackManager().scanPacks();
        client.reloadResources();
    }

    private static void downloadHttp(String urlStr, File dest) throws Exception {
        HttpURLConnection conn = (HttpURLConnection) new URL(urlStr).openConnection();
        conn.setRequestProperty("User-Agent", "Paraguacraft-Modern/1.0");
        conn.setConnectTimeout(12000);
        conn.setReadTimeout(60000);
        int total = conn.getContentLength();
        try (InputStream in = conn.getInputStream(); FileOutputStream out = new FileOutputStream(dest)) {
            byte[] buf = new byte[8192];
            int read;
            int done = 0;
            while ((read = in.read(buf)) >= 0) {
                out.write(buf, 0, read);
                done += read;
            }
            if (total > 0 && done < total / 2) {
                throw new IllegalStateException("Descarga incompleta");
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
