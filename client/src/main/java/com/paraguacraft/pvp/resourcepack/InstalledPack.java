package com.paraguacraft.pvp.resourcepack;

public final class InstalledPack {
    public final String fileName;
    public final String displayName;
    public final long sizeBytes;
    public final boolean active;

    public InstalledPack(String fileName, String displayName, long sizeBytes, boolean active) {
        this.fileName = fileName;
        this.displayName = displayName;
        this.sizeBytes = sizeBytes;
        this.active = active;
    }
}
