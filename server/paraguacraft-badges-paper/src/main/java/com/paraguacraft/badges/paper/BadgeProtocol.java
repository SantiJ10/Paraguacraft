package com.paraguacraft.badges.paper;

/**
 * Protocolo del canal plugin message {@code paraguacraft:bdg}, compartido con
 * {@code BadgeProtocol} de los clientes PvP 1.8.9 y Modern.
 */
public final class BadgeProtocol {

    public static final String CHANNEL = "paraguacraft:bdg";

    public static final byte C2S_REGISTER = 0x01;
    public static final byte S2C_SYNC = 0x01;
    public static final byte S2C_UPDATE = 0x02;

    public static final byte BADGE_NONE = 0;
    public static final byte BADGE_PARAGUACRAFT = 1;
    public static final byte BADGE_STAFF = 2;

    private BadgeProtocol() {}
}
