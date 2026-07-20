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

/** Servidores escritos por el launcher en `paraguacraft_servers.json`. */
public final class DefaultServers {

    public record Entry(String name, String address, String description) {}

    private static final int CURRENT_VERSION = 2;

    private DefaultServers() {}

    public static List<Entry> load(MinecraftClient client) {
        Path path = client.runDirectory.toPath().resolve("paraguacraft_servers.json");
        if (Files.isRegularFile(path)) {
            try {
                String json = Files.readString(path);
                JsonObject root = JsonParser.parseString(json).getAsJsonObject();
                int version = root.has("version") ? root.get("version").getAsInt() : 0;
                JsonArray arr = root.getAsJsonArray("servers");
                List<Entry> out = new ArrayList<>();
                for (var el : arr) {
                    JsonObject o = el.getAsJsonObject();
                    out.add(new Entry(
                        o.get("name").getAsString(),
                        o.get("address").getAsString(),
                        o.has("description") ? o.get("description").getAsString() : ""
                    ));
                }
                if (version >= CURRENT_VERSION && !out.isEmpty()) {
                    return normalize(out);
                }
            } catch (IOException | RuntimeException ignored) {
            }
        }
        return fallback();
    }

    /** Corrige entradas viejas (LibreCraft, IP incorrecta de CubeCraft). */
    private static List<Entry> normalize(List<Entry> entries) {
        List<Entry> out = new ArrayList<>(entries.size());
        for (Entry e : entries) {
            String name = e.name();
            String address = e.address();
            if ("LibreCraft".equalsIgnoreCase(name) || address.toLowerCase().contains("librecraft")) {
                name = "Regorland";
                address = "regorland.net";
            }
            if ("m.cubecraft.net".equalsIgnoreCase(address)) {
                address = "play.cubecraft.net";
            }
            out.add(new Entry(name, address, e.description()));
        }
        return out;
    }

    private static List<Entry> fallback() {
        return List.of(
            new Entry("Hypixel", "mc.hypixel.net", "BedWars · SkyWars"),
            new Entry("CubeCraft", "play.cubecraft.net", "EggWars · SkyWars"),
            new Entry("Regorland", "regorland.net", "Survival · PvP"),
            new Entry("Hylex", "original.hylex.net", "Minijuegos"),
            new Entry("MineLatino", "play.minelatino.net", "Comunidad latina")
        );
    }
}
