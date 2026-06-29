package com.paraguacraft.pvp.modules;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.reflect.TypeToken;
import net.minecraft.client.Minecraft;
import net.minecraft.util.EnumChatFormatting;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.io.Reader;
import java.io.Writer;
import java.lang.reflect.Type;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;

/**
 * Alertas de chat simples, manejadas por el comando /chat alerts.
 * Cuando llega un mensaje con una palabra clave: suena un "ding" y se resalta la linea.
 */
public final class ChatAlerts {

    public static boolean enabled = true;
    public static boolean sound = true;
    public static boolean highlight = true;
    public static String color = "YELLOW";

    private static final Set<String> WORDS = new LinkedHashSet<String>();
    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();
    private static final Type DATA_TYPE = new TypeToken<Data>() {}.getType();
    private static boolean loaded = false;

    private static final class Data {
        boolean enabled = true;
        boolean sound = true;
        boolean highlight = true;
        String color = "YELLOW";
        List<String> words = new ArrayList<String>();
    }

    private ChatAlerts() {}

    public static void ensureLoaded() {
        if (!loaded) {
            load();
        }
    }

    public static List<String> words() {
        ensureLoaded();
        return new ArrayList<String>(WORDS);
    }

    public static boolean add(String word) {
        ensureLoaded();
        if (word == null) {
            return false;
        }
        String w = word.trim().toLowerCase();
        if (w.isEmpty()) {
            return false;
        }
        boolean added = WORDS.add(w);
        if (added) {
            save();
        }
        return added;
    }

    public static boolean remove(String word) {
        ensureLoaded();
        if (word == null) {
            return false;
        }
        boolean removed = WORDS.remove(word.trim().toLowerCase());
        if (removed) {
            save();
        }
        return removed;
    }

    public static void clear() {
        ensureLoaded();
        WORDS.clear();
        save();
    }

    /** Devuelve la primera palabra clave contenida en el texto (ya en minusculas) o null. */
    public static String firstMatch(String plainLower) {
        ensureLoaded();
        if (plainLower == null || plainLower.isEmpty()) {
            return null;
        }
        for (String w : WORDS) {
            if (plainLower.contains(w)) {
                return w;
            }
        }
        return null;
    }

    public static EnumChatFormatting colorFmt() {
        try {
            EnumChatFormatting fmt = EnumChatFormatting.valueOf(color.trim().toUpperCase());
            return fmt.isColor() ? fmt : EnumChatFormatting.YELLOW;
        } catch (Exception e) {
            return EnumChatFormatting.YELLOW;
        }
    }

    public static boolean setColor(String c) {
        if (c == null) {
            return false;
        }
        try {
            EnumChatFormatting fmt = EnumChatFormatting.valueOf(c.trim().toUpperCase());
            if (!fmt.isColor()) {
                return false;
            }
            color = c.trim().toUpperCase();
            save();
            return true;
        } catch (Exception e) {
            return false;
        }
    }

    private static File file() {
        Minecraft mc = Minecraft.getMinecraft();
        File base = (mc == null) ? new File("paraguacraft") : new File(mc.mcDataDir, "paraguacraft");
        return new File(base, "chat_alerts.json");
    }

    public static void load() {
        loaded = true;
        try {
            File f = file();
            if (!f.exists()) {
                save();
                return;
            }
            try (Reader r = new InputStreamReader(new FileInputStream(f), StandardCharsets.UTF_8)) {
                Data d = GSON.fromJson(r, DATA_TYPE);
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
                                WORDS.add(w.trim().toLowerCase());
                            }
                        }
                    }
                }
            }
        } catch (Exception ignored) {
        }
    }

    public static void save() {
        try {
            File f = file();
            File parent = f.getParentFile();
            if (parent != null && !parent.exists()) {
                parent.mkdirs();
            }
            Data d = new Data();
            d.enabled = enabled;
            d.sound = sound;
            d.highlight = highlight;
            d.color = color;
            d.words = new ArrayList<String>(WORDS);
            try (Writer w = new OutputStreamWriter(new FileOutputStream(f), StandardCharsets.UTF_8)) {
                GSON.toJson(d, DATA_TYPE, w);
            }
        } catch (Exception ignored) {
        }
    }
}
