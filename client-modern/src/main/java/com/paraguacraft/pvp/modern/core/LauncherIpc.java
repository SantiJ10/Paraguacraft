package com.paraguacraft.pvp.modern.core;

import java.io.File;
import java.io.RandomAccessFile;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;

/** Lee stats del launcher via mmap (`game-overlay.dat`). */
public final class LauncherIpc {

    private static final int SIZE = 512;
    private static final byte[] MAGIC = {'P', 'G', 'I', 'P'};

    public static final class Snapshot {
        public float cpuPct;
        public float ramPct;
        public float gpuPct;
        public float tempC;
        public boolean musicPlaying;
        public String musicTitle = "";
        public String musicArtist = "";
        public String musicImageUrl = "";
        public boolean valid;
    }

    private static volatile Snapshot last = new Snapshot();

    private LauncherIpc() {}

    public static Snapshot get() {
        return last;
    }

    public static void poll() {
        String path = System.getenv("PARAGUACRAFT_OVERLAY_IPC");
        if (path == null || path.isEmpty()) {
            File fallback = new File(System.getenv("APPDATA") != null
                ? System.getenv("APPDATA") + "\\ParaguacraftLauncher\\game-overlay.dat"
                : System.getProperty("user.home") + "/.config/ParaguacraftLauncher/game-overlay.dat");
            path = fallback.getAbsolutePath();
        }
        File file = new File(path);
        if (!file.isFile() || file.length() < 32) {
            last.valid = false;
            return;
        }
        try (RandomAccessFile raf = new RandomAccessFile(file, "r")) {
            byte[] buf = new byte[SIZE];
            int read = raf.read(buf);
            if (read < 32) {
                last.valid = false;
                return;
            }
            for (int i = 0; i < 4; i++) {
                if (buf[i] != MAGIC[i]) {
                    last.valid = false;
                    return;
                }
            }
            ByteBuffer bb = ByteBuffer.wrap(buf).order(ByteOrder.LITTLE_ENDIAN);
            bb.position(8);
            Snapshot s = new Snapshot();
            s.cpuPct = bb.getFloat();
            s.ramPct = bb.getFloat();
            s.gpuPct = bb.getFloat();
            s.tempC = bb.getFloat();
            s.musicPlaying = bb.get(24) != 0;
            s.musicTitle = readUtf8(buf, 25, 128);
            s.musicArtist = readUtf8(buf, 153, 64);
            s.musicImageUrl = sanitizeUrl(readUtf8(buf, 217, 256));
            s.valid = true;
            last = s;
        } catch (Exception e) {
            last.valid = false;
        }
    }

    private static String readUtf8(byte[] buf, int off, int max) {
        int end = Math.min(buf.length, off + max);
        int len = 0;
        while (off + len < end && buf[off + len] != 0) {
            len++;
        }
        return new String(buf, off, len, StandardCharsets.UTF_8).trim();
    }

    private static String sanitizeUrl(String raw) {
        if (raw == null || raw.isEmpty()) {
            return "";
        }
        int https = raw.indexOf("https://");
        int http = raw.indexOf("http://");
        int file = raw.indexOf("file://");
        int start = https >= 0 ? https : (http >= 0 ? http : file);
        if (start < 0) {
            return "";
        }
        String url = raw.substring(start);
        int end = url.length();
        for (int i = 0; i < url.length(); i++) {
            char c = url.charAt(i);
            if (c <= 32 || c == 0) {
                end = i;
                break;
            }
        }
        return url.substring(0, end);
    }
}
