package com.paraguacraft.pvp.resourcepack;

import net.minecraft.client.Minecraft;
import net.minecraft.client.resources.ResourcePackRepository;

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
    public static final String BRAND_PACK = "ParaguacraftBrandPack";
    /** Debe coincidir con catalog.json / launcher PACK_189_SHA1. */
    public static final String OFFICIAL_SHA1 = "5a14ce92d1e8d2ef05b50421376ae0536c8736c6";
    private static final String PENDING_PACK = OFFICIAL_PACK + ".pending";
    private static final String[] OFFICIAL_URLS = new String[] {
        "https://raw.githubusercontent.com/SantiJ10/Paraguacraft/main/clientes/paraguacraft-pvp/packs/paraguacraft-pvp-189.zip",
        "https://github.com/SantiJ10/Paraguacraft/raw/main/clientes/paraguacraft-pvp/packs/paraguacraft-pvp-189.zip",
        "https://cdn.jsdelivr.net/gh/SantiJ10/Paraguacraft@main/clientes/paraguacraft-pvp/packs/paraguacraft-pvp-189.zip",
    };

    private static boolean officialRefreshStarted;

    public interface ProgressListener {
        void onProgress(String status, float ratio);
        void onComplete(String fileName);
        void onError(String message);
    }

    private ResourcePackManager() {}

    public static File packsDir() {
        return new File(Minecraft.getMinecraft().mcDataDir, "resourcepacks");
    }

    private static File packsDir(File gameDir) {
        return new File(gameDir, "resourcepacks");
    }

    /**
     * Antes de cargar texture packs (FML preInit): aplica .pending e instala el pack oficial
     * con el SHA embebido. En Windows el zip activo no se puede borrar in-game.
     */
    public static void bootstrapOfficialPack(File gameDir) {
        if (gameDir == null) {
            return;
        }
        File dir = packsDir(gameDir);
        if (!dir.exists() && !dir.mkdirs()) {
            System.err.println("[Paraguacraft] No se pudo crear resourcepacks/");
            return;
        }
        File dest = new File(dir, OFFICIAL_PACK);
        File pending = new File(dir, PENDING_PACK);
        applyPendingIfValid(pending, dest);
        if (dest.isFile() && sha1Matches(dest, OFFICIAL_SHA1)) {
            return;
        }
        System.out.println("[Paraguacraft] Actualizando pack oficial (SHA distinto o ausente)...");
        try {
            downloadOfficialTo(pending);
            if (!applyPendingIfValid(pending, dest)) {
                System.err.println("[Paraguacraft] No se pudo instalar " + OFFICIAL_PACK + " tras descargar");
            } else {
                System.out.println("[Paraguacraft] Pack oficial listo: " + OFFICIAL_PACK);
            }
        } catch (Exception e) {
            System.err.println("[Paraguacraft] Fallo al actualizar pack: " + e.getMessage());
        }
    }

    private static boolean applyPendingIfValid(File pending, File dest) {
        if (pending == null || !pending.isFile() || !sha1Matches(pending, OFFICIAL_SHA1)) {
            return dest != null && dest.isFile() && sha1Matches(dest, OFFICIAL_SHA1);
        }
        try {
            if (dest.exists() && !dest.delete()) {
                // Renombrar viejo y reintentar
                File bak = new File(dest.getParentFile(), OFFICIAL_PACK + ".old");
                if (bak.exists()) {
                    bak.delete();
                }
                if (!dest.renameTo(bak)) {
                    System.err.println("[Paraguacraft] ZIP oficial bloqueado; queda " + PENDING_PACK);
                    return false;
                }
                bak.delete();
            }
            if (!pending.renameTo(dest)) {
                copyFile(pending, dest);
                pending.delete();
            }
            return dest.isFile() && sha1Matches(dest, OFFICIAL_SHA1);
        } catch (Exception e) {
            System.err.println("[Paraguacraft] applyPending: " + e.getMessage());
            return false;
        }
    }

    /** Token 1.8.9: nombre exacto del zip o carpeta (sin prefijo file/). */
    public static String packToken(String fileName) {
        if (fileName == null) {
            return "";
        }
        if (fileName.startsWith("file/")) {
            return fileName.substring(5);
        }
        return fileName;
    }

    public static boolean isPackActive(String fileName) {
        return isPackLiveSelected(fileName) || isPackListedInOptions(fileName);
    }

    /** Pack realmente en Selected del repositorio (lo que ve el juego y la UI). */
    public static boolean isPackLiveSelected(String fileName) {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc == null || fileName == null) {
            return false;
        }
        ResourcePackRepository repo = mc.getResourcePackRepository();
        if (repo == null) {
            return false;
        }
        @SuppressWarnings("unchecked")
        List<ResourcePackRepository.Entry> selected = repo.getRepositoryEntries();
        if (selected == null) {
            return false;
        }
        for (ResourcePackRepository.Entry e : selected) {
            if (e == null) {
                continue;
            }
            if (entryMatches(e.getResourcePackName(), fileName)) {
                return true;
            }
        }
        return false;
    }

    private static boolean isPackListedInOptions(String fileName) {
        List<String> active = Minecraft.getMinecraft().gameSettings.resourcePacks;
        if (active == null || fileName == null) {
            return false;
        }
        for (String p : active) {
            if (entryMatches(p, fileName)) {
                return true;
            }
        }
        return false;
    }

    private static boolean entryMatches(String entryName, String fileName) {
        if (entryName == null || fileName == null) {
            return false;
        }
        String t = packToken(entryName);
        String token = packToken(fileName);
        if (t.equalsIgnoreCase(token)
            || t.equalsIgnoreCase(token.replace(".zip", ""))
            || (token.endsWith(".zip") && t.equalsIgnoreCase(token))) {
            return true;
        }
        if (token.contains("paraguacraft-pvp-189") && t.contains("paraguacraft-pvp-189")) {
            return true;
        }
        if (token.equalsIgnoreCase(BRAND_PACK) && t.equalsIgnoreCase(BRAND_PACK)) {
            return true;
        }
        return false;
    }

    public static List<InstalledPack> listInstalled() {
        List<InstalledPack> out = new ArrayList<InstalledPack>();
        File dir = packsDir();
        if (!dir.exists()) {
            dir.mkdirs();
            return out;
        }
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
            boolean enabled = isPackLiveSelected(name);
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

    /**
     * Aplica stack PvP: Brand (si existe) + oficial con máxima prioridad.
     * En 1.8.9 hay que llamar {@link ResourcePackRepository#setRepositories} como hace
     * el botón Aceptar de la UI; solo escribir {@code gameSettings.resourcePacks} no activa el pack.
     */
    public static void applyOfficialStack() {
        File official = new File(packsDir(), OFFICIAL_PACK);
        if (!official.isFile()) {
            return;
        }
        List<String> names = new ArrayList<String>();
        File brandDir = new File(packsDir(), BRAND_PACK);
        if (brandDir.isDirectory() || new File(packsDir(), BRAND_PACK + ".zip").isFile()) {
            names.add(BRAND_PACK);
        }
        names.add(OFFICIAL_PACK);
        applySelectedByNames(names);
    }

    public static void applyPack(String fileName) {
        if (OFFICIAL_PACK.equalsIgnoreCase(fileName)
            || (fileName != null && fileName.contains("paraguacraft-pvp-189"))) {
            applyOfficialStack();
            return;
        }
        List<String> names = new ArrayList<String>();
        names.add(packToken(fileName));
        applySelectedByNames(names);
    }

    /** Solo pack oficial activo (compat). */
    public static void applyOfficialPack() {
        applyOfficialStack();
    }

    /**
     * Garantiza el pack oficial en Selected (repositorio), no solo en options.txt.
     * Se puede llamar cada vez que se abre el menú principal.
     * Si el zip local está desactualizado (sin destroy_stage / SHA viejo), lo re-descarga.
     */
    public static void ensureOfficialSelected() {
        ensureOfficialFileCurrentAsync();
        File official = new File(packsDir(), OFFICIAL_PACK);
        if (!official.isFile()) {
            return;
        }
        if (isPackLiveSelected(OFFICIAL_PACK)) {
            // options desfasados respecto al repo → reescribir lista limpia
            normalizeOptionsFromLive();
            return;
        }
        applyOfficialStack();
    }

    /** Reemplaza el zip oficial si no coincide el SHA embebido (async; usa .pending si está bloqueado). */
    private static void ensureOfficialFileCurrentAsync() {
        File official = new File(packsDir(), OFFICIAL_PACK);
        if (official.isFile() && sha1Matches(official, OFFICIAL_SHA1)) {
            return;
        }
        if (officialRefreshStarted) {
            return;
        }
        officialRefreshStarted = true;
        new Thread(new Runnable() {
            @Override
            public void run() {
                try {
                    File pending = new File(packsDir(), PENDING_PACK);
                    downloadOfficialTo(pending);
                    Minecraft.getMinecraft().addScheduledTask(new Runnable() {
                        @Override
                        public void run() {
                            File dest = new File(packsDir(), OFFICIAL_PACK);
                            if (applyPendingIfValid(pending, dest)) {
                                applyOfficialStack();
                            } else {
                                System.err.println(
                                    "[Paraguacraft] Pack descargado a "
                                        + PENDING_PACK
                                        + "; se aplicará al reiniciar el juego."
                                );
                            }
                        }
                    });
                } catch (Exception e) {
                    System.err.println("[Paraguacraft] No se pudo actualizar " + OFFICIAL_PACK + ": " + e.getMessage());
                    officialRefreshStarted = false;
                }
            }
        }, "Paraguacraft-OfficialPack").start();
    }

    private static void downloadOfficialTo(File dest) throws Exception {
        File tmp = new File(dest.getParentFile(), dest.getName() + ".part");
        Exception last = null;
        for (String url : OFFICIAL_URLS) {
            try {
                downloadUrlToFile(url, tmp);
                if (!sha1Matches(tmp, OFFICIAL_SHA1)) {
                    tmp.delete();
                    last = new IllegalStateException("SHA1 no coincide tras descargar " + url);
                    continue;
                }
                if (dest.exists() && !dest.delete()) {
                    // dejar .part renombrable
                }
                if (dest.exists()) {
                    dest.delete();
                }
                if (!tmp.renameTo(dest)) {
                    copyFile(tmp, dest);
                    tmp.delete();
                }
                if (!sha1Matches(dest, OFFICIAL_SHA1)) {
                    throw new IllegalStateException("SHA invalid final " + dest.getName());
                }
                return;
            } catch (Exception e) {
                last = e;
                if (tmp.exists()) {
                    tmp.delete();
                }
            }
        }
        if (last != null) {
            throw last;
        }
    }

    private static void downloadUrlToFile(String urlStr, File dest) throws Exception {
        HttpURLConnection conn = (HttpURLConnection) new URL(urlStr).openConnection();
        conn.setInstanceFollowRedirects(true);
        conn.setRequestProperty("User-Agent", "Paraguacraft-Client/2.1 (Forge-1.8.9)");
        conn.setConnectTimeout(20000);
        conn.setReadTimeout(120000);
        int code = conn.getResponseCode();
        if (code < 200 || code >= 300) {
            throw new IllegalStateException("HTTP " + code + " " + urlStr);
        }
        try (InputStream in = conn.getInputStream(); FileOutputStream out = new FileOutputStream(dest)) {
            byte[] buf = new byte[8192];
            int read;
            while ((read = in.read(buf)) != -1) {
                out.write(buf, 0, read);
            }
        } finally {
            conn.disconnect();
        }
    }

    private static boolean sha1Matches(File file, String expected) {
        if (file == null || !file.isFile() || expected == null || expected.isEmpty()) {
            return false;
        }
        try {
            MessageDigest md = MessageDigest.getInstance("SHA-1");
            try (FileInputStream in = new FileInputStream(file)) {
                byte[] buf = new byte[8192];
                int read;
                while ((read = in.read(buf)) != -1) {
                    md.update(buf, 0, read);
                }
            }
            return toHex(md.digest()).equalsIgnoreCase(expected);
        } catch (Exception e) {
            return false;
        }
    }

    public static void clearActivePack() {
        applySelectedByNames(new ArrayList<String>());
    }

    public static void deletePack(String fileName) {
        File f = new File(packsDir(), fileName);
        if (f.exists()) {
            f.delete();
        }
        if (isPackLiveSelected(fileName) || isPackListedInOptions(fileName)) {
            clearActivePack();
        }
    }

    /**
     * Mueve packs a Selected del repositorio (como Done de la UI) y persiste options.
     * Orden: primero = menor prioridad, último = mayor (FallbackResourceManager, 1.8.9).
     */
    @SuppressWarnings("unchecked")
    private static void applySelectedByNames(List<String> orderedLowToHigh) {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc == null) {
            return;
        }
        ResourcePackRepository repo = mc.getResourcePackRepository();
        if (repo == null) {
            return;
        }
        repo.updateRepositoryEntriesAll();

        List<ResourcePackRepository.Entry> all = repo.getRepositoryEntriesAll();
        List<ResourcePackRepository.Entry> selected = new ArrayList<ResourcePackRepository.Entry>();
        List<String> names = new ArrayList<String>();

        if (orderedLowToHigh != null) {
            for (String wanted : orderedLowToHigh) {
                ResourcePackRepository.Entry found = findEntry(all, wanted);
                if (found == null) {
                    continue;
                }
                if (!selected.contains(found)) {
                    selected.add(found);
                    names.add(found.getResourcePackName());
                }
            }
        }

        mc.gameSettings.resourcePacks = names;
        mc.gameSettings.saveOptions();
        repo.setRepositories(selected);
        mc.refreshResources();
    }

    @SuppressWarnings("unchecked")
    private static void normalizeOptionsFromLive() {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc == null) {
            return;
        }
        ResourcePackRepository repo = mc.getResourcePackRepository();
        if (repo == null) {
            return;
        }
        List<ResourcePackRepository.Entry> selected = repo.getRepositoryEntries();
        if (selected == null) {
            return;
        }
        List<String> names = new ArrayList<String>();
        for (ResourcePackRepository.Entry e : selected) {
            if (e != null && e.getResourcePackName() != null) {
                names.add(e.getResourcePackName());
            }
        }
        List<String> current = mc.gameSettings.resourcePacks;
        if (current != null && current.equals(names)) {
            return;
        }
        mc.gameSettings.resourcePacks = names;
        mc.gameSettings.saveOptions();
    }

    private static ResourcePackRepository.Entry findEntry(
        List<ResourcePackRepository.Entry> all,
        String fileName
    ) {
        if (all == null || fileName == null) {
            return null;
        }
        for (ResourcePackRepository.Entry e : all) {
            if (e == null) {
                continue;
            }
            if (entryMatches(e.getResourcePackName(), fileName)) {
                return e;
            }
        }
        return null;
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
