package com.paraguacraft.pvp.resourcepack;

/** Metadatos de un archivo remoto a descargar. */
public final class PackDownload {
    public final String url;
    public final String fileName;
    public final long sizeBytes;
    public final String sha1;

    public PackDownload(String url, String fileName, long sizeBytes, String sha1) {
        this.url = url;
        this.fileName = fileName;
        this.sizeBytes = sizeBytes;
        this.sha1 = sha1;
    }
}
