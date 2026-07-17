package com.paraguacraft.pvp.modern.resourcepack;

public final class CatalogPack {

    public final String id;
    public final String title;
    public final String subtitle;
    public final String downloadUrl;
    public final String fileName;
    public final String sha1;
    public final String badge;

    public CatalogPack(
        String id,
        String title,
        String subtitle,
        String downloadUrl,
        String fileName,
        String sha1,
        String badge
    ) {
        this.id = id;
        this.title = title;
        this.subtitle = subtitle;
        this.downloadUrl = downloadUrl;
        this.fileName = fileName;
        this.sha1 = sha1;
        this.badge = badge;
    }
}
