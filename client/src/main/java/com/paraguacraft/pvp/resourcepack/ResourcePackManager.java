package com.paraguacraft.pvp.resourcepack;

import net.minecraft.client.Minecraft;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.channels.FileChannel;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.List;

public final class ResourcePackManager {

    public static final String OFFICIAL_PACK = "paraguacraft-pvp-189.zip";

    public interface ProgressListener {
        void onProgress(String status, float ratio);
        void onComplete(String fileName);
        void onError(String message);
    }

    private ResourcePackManager() {}

    public static File packsDir() {
        return new File(Minecraft.getMinecraft().mcDataDir, "resourcepacks");
    }

    public static List<InstalledPack> listInstalled() {
        List<InstalledPack> out = new ArrayList<InstalledPack>();
        File dir = packsDir();
        if (!dir.exists()) {
            dir.mkdirs();
            return out;
        }
        List<String> active = Minecraft.getMinecraft().gameSettings.resourcePacks;
        File[] files = dir.listFiles();
        if (files == null) {
            return out;
        }
        for (File f : files) {
            if (f.isDirectory()) {
                continue;
            }
            String name = f.getName();
            if (!name.toLowerCase().endsWith(".zip")) {
                continue;
            }
            String token = "file/" + name;
            boolean enabled = active != null && active.contains(token);
            out.add(new InstalledPack(name, stripExtension(name), f.length(), enabled));
        }
        out.sort((a, b) -> a.displayName.compareToIgnoreCase(b.displayName));
        return out;
    }

    public static String importFile(File source) throws Exception {
        if (source == null || !source.exists()) {
            throw new IllegalArgumentException("Archivo inválido");
        }
        String name = source.getName();
        if (!name.toLowerCase().endsWith(".zip")) {
            throw new IllegalArgumentException("Solo se admiten archivos .zip");
        }
        File dir = packsDir();
        if (!dir.exists()) {
            dir.mkdirs();
        }
        File dest = uniqueDest(dir, name);
        copyFile(source, dest);
        return dest.getName();
    }

    public static void applyPack(String fileName) {
        if (OFFICIAL_PACK.equalsIgnoreCase(fileName)) {
            purgeNonOfficialPacks();
        }
        Minecraft mc = Minecraft.getMinecraft();
        List<String> packs = new ArrayList<String>();
        packs.add("file/" + fileName);
        mc.gameSettings.resourcePacks = packs;
        mc.gameSettings.saveOptions();
        mc.getResourcePackRepository().updateRepositoryEntriesAll();
        mc.refreshResources();
    }

    private static void purgeNonOfficialPacks() {
        File dir = packsDir();
        File[] files = dir.listFiles();
        if (files == null) {
            return;
        }
        for (File f : files) {
            if (f.isDirectory()) {
                continue;
            }
            if (!f.isFile()) {
                continue;
            }
            String name = f.getName();
            if (OFFICIAL_PACK.equalsIgnoreCase(name)) {
                continue;
            }
            if (name.toLowerCase().endsWith(".zip")) {
                f.delete();
            }
        }
    }

    /** Solo pack oficial activo (prioridad sobre vanilla en 1.8.9). */
    public static void applyOfficialPack() {
        purgeNonOfficialPacks();
        if (!new File(packsDir(), OFFICIAL_PACK).isFile()) {
            return;
        }
        applyPack(OFFICIAL_PACK);
    }

    public static void clearActivePack() {
        Minecraft mc = Minecraft.getMinecraft();
        mc.gameSettings.resourcePacks = new ArrayList<String>();
        mc.gameSettings.saveOptions();
        mc.getResourcePackRepository().updateRepositoryEntriesAll();
        mc.refreshResources();
    }

    public static void deletePack(String fileName) {
        File f = new File(packsDir(), fileName);
        if (f.exists()) {
            f.delete();
        }
        List<String> active = Minecraft.getMinecraft().gameSettings.resourcePacks;
        if (active != null && active.contains("file/" + fileName)) {
            clearActivePack();
        }
    }

    public static void downloadCatalogPack(CatalogPack pack, ProgressListener listener) {
        new Thread(new Runnable() {
            @Override
            public void run() {
                try {
                    String name = pack.fileName != null ? pack.fileName : pack.id + ".zip";
                    Exception last = null;
                    if (pack.usesDirectUrl()) {
                        try {
                            listener.onProgress("Descargando desde GitHub…", 0.05f);
                            downloadHttp(pack.downloadUrl, name, pack.sha1, listener);
                            return;
                        } catch (Exception e) {
                            last = e;
                        }
                    }
                    String driveFolder = CatalogLoader.getDriveFolderUrl();
                    if (driveFolder != null && !driveFolder.isEmpty()) {
                        try {
                            listener.onProgress("Descargando desde Google Drive…", 0.08f);
                            downloadDrive(driveFolder, name, pack.sha1, listener);
                            return;
                        } catch (Exception e) {
                            last = e;
                        }
                    }
                    if (pack.hasFallbackUrl()) {
                        listener.onProgress("Reintentando mirror…", 0.08f);
                        downloadHttp(pack.fallbackDownloadUrl, name, pack.sha1, listener);
                        return;
                    }
                    if (last != null) {
                        throw last;
                    }
                    throw new IllegalStateException("Pack sin fuente de descarga: " + pack.id);
                } catch (Exception e) {
                    listener.onError(e.getMessage() != null ? e.getMessage() : "Error de descarga");
                }
            }
        }, "Paraguacraft-PackDL").start();
    }

    private static void downloadHttp(
        String url,
        String fileName,
        String sha1,
        ProgressListener listener
    ) throws Exception {
        PackDownload info = new PackDownload(url, fileName, 0L, sha1);
        listener.onProgress("Descargando " + fileName + "…", 0.1f);
        File saved = downloadToPacks(info, listener, null);
        applyOnMainThread(saved.getName());
        listener.onComplete(saved.getName());
    }

    private static void downloadDrive(
        String folderUrl,
        String fileName,
        String sha1,
        ProgressListener listener
    ) throws Exception {
        listener.onProgress("Descargando " + fileName + "…", 0.1f);
        HttpURLConnection conn = DriveClient.openDownload(folderUrl, fileName);
        PackDownload info = new PackDownload(
            conn.getURL().toString(),
            fileName,
            conn.getContentLength(),
            sha1
        );
        File saved = downloadToPacks(info, listener, conn);
        applyOnMainThread(saved.getName());
        listener.onComplete(saved.getName());
    }

    private static File downloadToPacks(
        PackDownload info,
        ProgressListener listener,
        HttpURLConnection existing
    ) throws Exception {
        File dir = packsDir();
        if (!dir.exists()) {
            dir.mkdirs();
        }
        File dest = uniqueDest(dir, info.fileName);
        HttpURLConnection conn = existing;
        boolean ownConn = conn == null;
        if (conn == null) {
            conn = (HttpURLConnection) new URL(info.url).openConnection();
            conn.setRequestProperty("User-Agent", "Paraguacraft-Client/2.0 (Forge-1.8.9)");
            conn.setConnectTimeout(20000);
            conn.setReadTimeout(120000);
            int code = conn.getResponseCode();
            if (code < 200 || code >= 300) {
                throw new IllegalStateException("CDN HTTP " + code);
            }
        }
        long total = info.sizeBytes > 0 ? info.sizeBytes : conn.getContentLength();
        MessageDigest sha1 = info.sha1 != null ? MessageDigest.getInstance("SHA-1") : null;
        try (InputStream in = conn.getInputStream();
             FileOutputStream out = new FileOutputStream(dest)) {
            byte[] buf = new byte[8192];
            long done = 0;
            int read;
            while ((read = in.read(buf)) != -1) {
                out.write(buf, 0, read);
                if (sha1 != null) {
                    sha1.update(buf, 0, read);
                }
                done += read;
                if (total > 0) {
                    float ratio = 0.1f + 0.85f * (done / (float) total);
                    listener.onProgress("Descargando…", Math.min(0.95f, ratio));
                }
            }
        } finally {
            if (ownConn) {
                conn.disconnect();
            }
        }
        if (sha1 != null && info.sha1 != null) {
            String hex = toHex(sha1.digest());
            if (!hex.equalsIgnoreCase(info.sha1)) {
                dest.delete();
                throw new IllegalStateException("SHA1 no coincide");
            }
        }
        return dest;
    }

    private static void applyOnMainThread(final String fileName) {
        Minecraft.getMinecraft().addScheduledTask(new Runnable() {
            @Override
            public void run() {
                applyPack(fileName);
            }
        });
    }

    private static File uniqueDest(File dir, String name) {
        File dest = new File(dir, name);
        if (!dest.exists()) {
            return dest;
        }
        String base = stripExtension(name);
        String ext = name.substring(base.length());
        int i = 2;
        while (dest.exists()) {
            dest = new File(dir, base + " (" + i + ")" + ext);
            i++;
        }
        return dest;
    }

    private static void copyFile(File src, File dest) throws Exception {
        try (FileInputStream in = new FileInputStream(src);
             FileOutputStream out = new FileOutputStream(dest);
             FileChannel inCh = in.getChannel();
             FileChannel outCh = out.getChannel()) {
            outCh.transferFrom(inCh, 0, inCh.size());
        }
    }

    private static String stripExtension(String name) {
        int dot = name.lastIndexOf('.');
        return dot > 0 ? name.substring(0, dot) : name;
    }

    private static String toHex(byte[] bytes) {
        StringBuilder sb = new StringBuilder(bytes.length * 2);
        for (byte b : bytes) {
            sb.append(String.format("%02x", b & 0xff));
        }
        return sb.toString();
    }
}
