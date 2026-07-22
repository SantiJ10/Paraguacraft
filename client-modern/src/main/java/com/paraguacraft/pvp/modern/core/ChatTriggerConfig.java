package com.paraguacraft.pvp.modern.core;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.reflect.TypeToken;
import net.fabricmc.loader.api.FabricLoader;

import java.io.IOException;
import java.io.Reader;
import java.io.Writer;
import java.lang.reflect.Type;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

/** Reglas configurables de alertas en chat (BedWars / Hypixel). */
public final class ChatTriggerConfig {

    public static final class Rule {
        public String id = "rule";
        public boolean enabled = true;
        public String keywords = "";
        public String title = "";
        public String color = "RED";
        public int fadeIn = 10;
        public int stay = 40;
        public int fadeOut = 10;
        public boolean hypixelOnly = false;

        public Rule() {}

        public Rule(String id, String keywords, String title, String color) {
            this.id = id;
            this.keywords = keywords;
            this.title = title;
            this.color = color;
        }
    }

    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();
    private static final Type LIST_TYPE = new TypeToken<List<Rule>>() {}.getType();

    private static List<Rule> rules = defaultRules();
    private static boolean loaded;

    private ChatTriggerConfig() {}

    public static List<Rule> getRules() {
        ensureLoaded();
        return rules;
    }

    public static void ensureLoaded() {
        if (loaded) {
            return;
        }
        load();
    }

    public static void load() {
        loaded = true;
        Path file = configFile();
        if (!Files.isRegularFile(file)) {
            rules = defaultRules();
            save();
            return;
        }
        try (Reader reader = Files.newBufferedReader(file, StandardCharsets.UTF_8)) {
            List<Rule> parsed = GSON.fromJson(reader, LIST_TYPE);
            rules = parsed == null || parsed.isEmpty() ? defaultRules() : parsed;
        } catch (Exception ignored) {
            rules = defaultRules();
        }
    }

    public static void save() {
        try {
            Path file = configFile();
            Files.createDirectories(file.getParent());
            try (Writer writer = Files.newBufferedWriter(file, StandardCharsets.UTF_8)) {
                GSON.toJson(rules, LIST_TYPE, writer);
            }
        } catch (IOException ignored) {
        }
    }

    private static Path configFile() {
        return FabricLoader.getInstance().getGameDir().resolve("paraguacraft/chat_triggers.json");
    }

    public static List<Rule> defaultRules() {
        List<Rule> out = new ArrayList<>();
        out.add(new Rule(
            "bed_destroyed",
            "bed was destroyed,cama fue destruida,cama ha sido destruida,bed destroyed",
            "CAMA DESTRUIDA",
            "RED"
        ));
        Rule bedAttack = new Rule(
            "bed_attacked",
            "bed is being attacked,cama esta siendo atacada,your bed is being broken",
            "CAMA EN PELIGRO",
            "GOLD"
        );
        bedAttack.stay = 30;
        out.add(bedAttack);
        Rule finalKill = new Rule(
            "final_kill",
            "final kill,asesinato final",
            "FINAL KILL",
            "DARK_RED"
        );
        finalKill.stay = 20;
        finalKill.hypixelOnly = true;
        out.add(finalKill);
        Rule trapped = new Rule("trapped", "trapped,atrapado", "TRAMPA", "DARK_RED");
        out.add(trapped);
        Rule bridge = new Rule("bridge", "bridge,puente", "BRIDGE", "AQUA");
        out.add(bridge);
        return out;
    }

    public static List<String> keywords(Rule rule) {
        List<String> out = new ArrayList<>();
        if (rule == null || rule.keywords == null) {
            return out;
        }
        for (String part : rule.keywords.split(",")) {
            String trimmed = part.trim().toLowerCase();
            if (!trimmed.isEmpty()) {
                out.add(trimmed);
            }
        }
        return out;
    }
}
