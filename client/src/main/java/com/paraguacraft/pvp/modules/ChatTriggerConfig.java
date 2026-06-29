package com.paraguacraft.pvp.modules;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.reflect.TypeToken;
import net.minecraft.client.Minecraft;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.lang.reflect.Type;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

/** Reglas configurables de alertas en chat (estilo ChatTriggers simplificado). */
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
    private static boolean loaded = false;

    private ChatTriggerConfig() {}

    public static List<Rule> getRules() {
        ensureLoaded();
        return rules;
    }

    public static void setRules(List<Rule> next) {
        rules = next == null || next.isEmpty() ? defaultRules() : next;
        save();
    }

    public static void resetDefaults() {
        rules = defaultRules();
        save();
    }

    public static void ensureLoaded() {
        if (loaded) {
            return;
        }
        load();
    }

    private static File configFile() {
        return new File(Minecraft.getMinecraft().mcDataDir, "paraguacraft/chat_triggers.json");
    }

    public static void load() {
        loaded = true;
        try {
            File file = configFile();
            if (!file.exists()) {
                rules = defaultRules();
                save();
                return;
            }
            try (InputStreamReader reader = new InputStreamReader(new FileInputStream(file), StandardCharsets.UTF_8)) {
                List<Rule> parsed = GSON.fromJson(reader, LIST_TYPE);
                rules = parsed == null || parsed.isEmpty() ? defaultRules() : parsed;
            }
        } catch (Exception ignored) {
            rules = defaultRules();
        }
    }

    public static void save() {
        try {
            File file = configFile();
            File parent = file.getParentFile();
            if (parent != null && !parent.exists()) {
                parent.mkdirs();
            }
            try (OutputStreamWriter writer = new OutputStreamWriter(new FileOutputStream(file), StandardCharsets.UTF_8)) {
                GSON.toJson(rules, LIST_TYPE, writer);
            }
        } catch (Exception ignored) {
        }
    }

    public static List<Rule> defaultRules() {
        List<Rule> out = new ArrayList<Rule>();
        out.add(new Rule(
            "bed_destroyed",
            "bed was destroyed,cama fue destruida,cama ha sido destruida",
            "CAMA DESTRUIDA",
            "RED"
        ));
        Rule bedAttack = new Rule(
            "bed_attacked",
            "bed is being attacked,cama esta siendo atacada,cama está siendo atacada,your bed is being broken,tu cama esta siendo",
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
        finalKill.fadeOut = 5;
        finalKill.hypixelOnly = true;
        out.add(finalKill);
        return out;
    }

    public static List<String> keywords(Rule rule) {
        List<String> out = new ArrayList<String>();
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

    public static Rule copy(Rule source) {
        Rule copy = new Rule();
        copy.id = source.id;
        copy.enabled = source.enabled;
        copy.keywords = source.keywords;
        copy.title = source.title;
        copy.color = source.color;
        copy.fadeIn = source.fadeIn;
        copy.stay = source.stay;
        copy.fadeOut = source.fadeOut;
        copy.hypixelOnly = source.hypixelOnly;
        return copy;
    }

    public static List<Rule> cloneRules() {
        List<Rule> out = new ArrayList<Rule>();
        for (Rule rule : getRules()) {
            out.add(copy(rule));
        }
        return out;
    }
}
