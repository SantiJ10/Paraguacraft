package com.paraguacraft.pvp.core;

import net.minecraft.client.Minecraft;
import net.minecraft.client.resources.I18n;

/** Traducciones del mod con soporte es_AR / es_ES / es_* y en_US. */
public final class ModLang {

    private static final String[][] ES_FALLBACK = {
        {"paraguacraft.hud.bw.iron", "Hierro"},
        {"paraguacraft.hud.bw.gold", "Oro"},
        {"paraguacraft.hud.bw.emerald", "Esmeralda"},
        {"paraguacraft.hud.bw.diamond", "Diamante"},
        {"paraguacraft.menu.on", "SI"},
        {"paraguacraft.menu.off", "NO"},
    };

    private ModLang() {}

    public static String format(String key, Object... args) {
        String value = args.length == 0 ? I18n.format(key) : I18n.format(key, args);
        if (!isSpanish()) {
            return value;
        }
        if (value.equals(key) || value.startsWith("paraguacraft.")) {
            String fb = fallbackEs(key);
            if (fb != null) {
                return args.length == 0 ? fb : String.format(fb.replace("%s", "%1$s"), args);
            }
        }
        return value;
    }

    public static String format(String key) {
        return format(key, new Object[0]);
    }

    public static boolean isSpanish() {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc == null || mc.getLanguageManager() == null
            || mc.getLanguageManager().getCurrentLanguage() == null) {
            return false;
        }
        String code = mc.getLanguageManager().getCurrentLanguage().getLanguageCode();
        return code != null && code.toLowerCase().startsWith("es");
    }

    private static String fallbackEs(String key) {
        for (String[] row : ES_FALLBACK) {
            if (row[0].equals(key)) {
                return row[1];
            }
        }
        return null;
    }
}
