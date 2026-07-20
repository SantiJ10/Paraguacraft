package com.paraguacraft.pvp.modern.network;

/** Canal plugin message compartido con ParaguacraftBadges (Paper). Mismo protocolo que el cliente 1.8.9. */
public final class BadgeProtocol {

    /** Max 20 chars (compat Bukkit). */
    public static final String CHANNEL_NAMESPACE = "paraguacraft";
    public static final String CHANNEL_PATH = "bdg";

    public static final byte C2S_REGISTER = 0x01;
    public static final byte S2C_SYNC = 0x01;
    public static final byte S2C_UPDATE = 0x02;

    public static final byte BADGE_NONE = 0;
    public static final byte BADGE_PARAGUACRAFT = 1;
    public static final byte BADGE_STAFF = 2;

    private BadgeProtocol() {}
}
