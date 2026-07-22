package com.paraguacraft.pvp.modern.core;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.reflect.TypeToken;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.util.Formatting;

import java.io.IOException;
import java.io.Reader;
import java.io.Writer;
import java.lang.reflect.Type;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Locale;
import java.util.Set;

/** Resalta palabras clave en chat con sonido opcional. */
public final class ChatAlerts {

    public static boolean enabled = true;
    public static boolean sound = true;
    public static boolean highlight = true;
    public static String color = "YELLOW";

    private static final Set<String> WORDS = new LinkedHashSet<>();
    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();
    private static final Type DATA_TYPE = new TypeToken<Data>() {}.getType();
    private static boolean loaded;

    private static final class Data {
        boolean enabled = true;
        boolean sound = true;
        boolean highlight = true;
        String color = "YELLOW";
        List<String> words = new ArrayList<>();
    }

    private ChatAlerts() {}

    public static void ensureLoaded() {
        if (!loaded) {
            load();
        }
    }

    public static String firstMatch(String plainLower) {
        ensureLoaded();
        if (!enabled || plainLower == null) {
            return null;
        }
        for (String w : WORDS) {
            if (plainLower.contains(w)) {
                return w;
            }
        }
        return null;
    }

    public static Formatting colorFmt() {
        try {
            Formatting fmt = Formatting.valueOf(color.trim().toUpperCase(Locale.ROOT));
            return fmt.isColor() ? fmt : Formatting.YELLOW;
        } catch (Exception e) {
            return Formatting.YELLOW;
        }
    }

    public static void load() {
        loaded = true;
        Path file = configFile();
        if (!Files.isRegularFile(file)) {
            seedDefaults();
            save();
            return;
        }
        try (Reader reader = Files.newBufferedReader(file, StandardCharsets.UTF_8)) {
            Data d = GSON.fromJson(reader, DATA_TYPE);
            if (d != null) {
                enabled = d.enabled;
                sound = d.sound;
                highlight = d.highlight;
                if (d.color != null) {
                    color = d.color;
                }
                WORDS.clear();
                if (d.words != null) {
                    for (String w : d.words) {
                        if (w != null && !w.trim().isEmpty()) {
                            WORDS.add(w.trim().toLowerCase(Locale.ROOT));
                        }
                    }
                }
            }
        } catch (Exception ignored) {
            seedDefaults();
        }
    }

    public static void resetDefaults() {
        seedDefaults();
        save();
    }

    private static void seedDefaults() {
        WORDS.clear();
        WORDS.add("gg");
        WORDS.add("bed");
        WORDS.add("final kill");
        WORDS.add("bridge");
    }

    public static void save() {
        try {
            Path file = configFile();
            Files.createDirectories(file.getParent());
            Data d = new Data();
            d.enabled = enabled;
            d.sound = sound;
            d.highlight = highlight;
            d.color = color;
            d.words = new ArrayList<>(WORDS);
            try (Writer writer = Files.newBufferedWriter(file, StandardCharsets.UTF_8)) {
                GSON.toJson(d, DATA_TYPE, writer);
            }
        } catch (IOException ignored) {
        }
    }

    private static Path configFile() {
        return FabricLoader.getInstance().getGameDir().resolve("paraguacraft/chat_alerts.json");
    }
}
