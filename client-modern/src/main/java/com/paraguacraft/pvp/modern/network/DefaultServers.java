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

    private DefaultServers() {}

    public static List<Entry> load(MinecraftClient client) {
        Path path = client.runDirectory.toPath().resolve("paraguacraft_servers.json");
        if (Files.isRegularFile(path)) {
            try {
                String json = Files.readString(path);
                JsonObject root = JsonParser.parseString(json).getAsJsonObject();
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
                if (!out.isEmpty()) {
                    return out;
                }
            } catch (IOException | RuntimeException ignored) {
            }
        }
        return fallback();
    }

    private static List<Entry> fallback() {
        return List.of(
            new Entry("Hypixel", "mc.hypixel.net", "BedWars · SkyWars"),
            new Entry("CubeCraft", "m.cubecraft.net", "EggWars · SkyWars"),
            new Entry("LibreCraft", "librecraft.gg", "Survival · PvP"),
            new Entry("Hylex", "hylex.net", "Minijuegos"),
            new Entry("MineLatino", "play.minelatino.net", "Comunidad latina")
        );
    }
}
