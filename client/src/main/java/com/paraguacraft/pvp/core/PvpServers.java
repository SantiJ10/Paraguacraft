package com.paraguacraft.pvp.core;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

/**
 * Servidores de redirección rápida (solo cliente PvP 1.8.9).
 * UniversoCraft y Mush reemplazan a Hylex / MineLatino.
 */
public final class PvpServers {

    public static final class Entry {
        public final String name;
        public final String address;

        public Entry(String name, String address) {
            this.name = name;
            this.address = address;
        }
    }

    private PvpServers() {}

    public static List<Entry> list() {
        List<Entry> out = new ArrayList<Entry>();
        out.add(new Entry("Hypixel", "mc.hypixel.net"));
        out.add(new Entry("CubeCraft", "play.cubecraft.net"));
        out.add(new Entry("Regorland", "regorland.net"));
        out.add(new Entry("UniversoCraft", "mc.universocraft.net"));
        out.add(new Entry("Mush", "mush.com.br"));
        return out;
    }

    /** Normaliza IPs legacy de configs antiguas. */
    public static String normalizeAddress(String address) {
        if (address == null) {
            return "";
        }
        String lower = address.toLowerCase(Locale.ROOT);
        if (lower.contains("hylex")) {
            return "mc.universocraft.net";
        }
        if (lower.contains("minelatino")) {
            return "mush.com.br";
        }
        if ("m.cubecraft.net".equals(lower)) {
            return "play.cubecraft.net";
        }
        return address;
    }
}
