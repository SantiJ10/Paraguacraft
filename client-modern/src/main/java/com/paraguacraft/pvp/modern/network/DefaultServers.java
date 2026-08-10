package com.paraguacraft.pvp.modern.network;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import net.minecraft.client.MinecraftClient;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

/** Servidores escritos por el launcher en `paraguacraft_servers.json`. */
public final class DefaultServers {

    public record Entry(String name, String address, String description, String note) {
        public Entry(String name, String address, String description) {
            this(name, address, description, "");
        }
    }

    /** v4: Minemen + notas offline/premium. */
    private static final int CURRENT_VERSION = 4;

    private DefaultServers() {}

    public static String offlineNote = "";

    public static List<Entry> load(MinecraftClient client) {
        Path path = client.runDirectory.toPath().resolve("paraguacraft_servers.json");
        if (Files.isRegularFile(path)) {
            try {
                String json = Files.readString(path);
                JsonObject root = JsonParser.parseString(json).getAsJsonObject();
                int version = root.has("version") ? root.get("version").getAsInt() : 0;
                if (root.has("offlineNote")) {
                    offlineNote = root.get("offlineNote").getAsString();
                }
                JsonArray arr = root.getAsJsonArray("servers");
                List<Entry> out = new ArrayList<>();
                for (var el : arr) {
                    JsonObject o = el.getAsJsonObject();
                    out.add(new Entry(
                        o.get("name").getAsString(),
                        o.get("address").getAsString(),
                        o.has("description") ? o.get("description").getAsString() : "",
                        o.has("note") ? o.get("note").getAsString() : ""
                    ));
                }
                if (version >= CURRENT_VERSION && !out.isEmpty()) {
                    return normalize(out);
                }
            } catch (IOException | RuntimeException ignored) {
            }
        }
        offlineNote =
            "Algunos servidores (Hypixel, Minemen, CubeCraft) exigen cuenta premium. Offline solo donde el server lo permite.";
        return fallback();
    }

    private static List<Entry> normalize(List<Entry> entries) {
        List<Entry> out = new ArrayList<>(entries.size());
        boolean hasMinemen = false;
        for (Entry e : entries) {
            String name = e.name();
            String address = e.address();
            String lower = address.toLowerCase(Locale.ROOT);
            String nameLower = name.toLowerCase(Locale.ROOT);
            if ("LibreCraft".equalsIgnoreCase(name) || lower.contains("librecraft")) {
                name = "Regorland";
                address = "regorland.net";
            }
            if ("m.cubecraft.net".equalsIgnoreCase(address)) {
                address = "play.cubecraft.net";
            }
            if (nameLower.contains("hylex") || lower.contains("hylex")) {
                name = "UniversoCraft";
                address = "mc.universocraft.net";
            }
            if (nameLower.contains("minelatino") || lower.contains("minelatino")) {
                name = "Mush";
                address = "mush.com.br";
            }
            if (nameLower.contains("minemen") || lower.contains("minemen")) {
                hasMinemen = true;
            }
            out.add(new Entry(name, address, e.description(), e.note()));
        }
        if (!hasMinemen) {
            out.add(1, new Entry(
                "Minemen Club",
                "na.minemen.club",
                "Practice · Duels · Pots",
                "Premium · anticheat estricto"
            ));
        }
        return out;
    }

    private static List<Entry> fallback() {
        return List.of(
            new Entry("Hypixel", "mc.hypixel.net", "BedWars · SkyWars · Duels", "Premium (Microsoft)"),
            new Entry("Minemen Club", "na.minemen.club", "Practice · Duels", "Premium · AC estricto"),
            new Entry("CubeCraft", "play.cubecraft.net", "EggWars · SkyWars · Lucky", "Premium"),
            new Entry("UniversoCraft", "mc.universocraft.net", "SkyWars · BedWars LATAM", ""),
            new Entry("Mush", "mush.com.br", "PvP · BedWars BR", ""),
            new Entry("Regorland", "regorland.net", "Survival · PvP latino", "A menudo no-premium OK")
        );
    }
}
