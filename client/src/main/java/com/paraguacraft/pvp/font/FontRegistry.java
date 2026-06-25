package com.paraguacraft.pvp.font;

import net.minecraft.client.Minecraft;
import net.minecraft.util.ResourceLocation;

import java.awt.Font;
import java.awt.FontFormatException;
import java.io.IOException;
import java.io.InputStream;

/**
 * Fuentes globales del cliente. Intenta cargar Montserrat desde assets;
 * si no existe, usa Segoe UI / SansSerif del sistema.
 */
public final class FontRegistry {

    private static CustomFontRenderer title;
    private static CustomFontRenderer regular;
    private static CustomFontRenderer small;
    private static boolean initialized;

    private FontRegistry() {}

    public static void init() {
        if (initialized) {
            return;
        }
        Font base = loadBaseFont();
        title = new CustomFontRenderer(new GlyphPage(base.deriveFont(Font.BOLD, 28f)), 1.0f);
        regular = new CustomFontRenderer(new GlyphPage(base.deriveFont(Font.PLAIN, 18f)), 1.0f);
        small = new CustomFontRenderer(new GlyphPage(base.deriveFont(Font.PLAIN, 14f)), 1.0f);
        initialized = true;
        System.out.println("[Paraguacraft V2] CFont cargado (" + base.getFontName() + ")");
    }

    private static Font loadBaseFont() {
        ResourceLocation montserrat = new ResourceLocation("paraguacraft", "fonts/Montserrat-Regular.ttf");
        ResourceLocation roboto = new ResourceLocation("paraguacraft", "fonts/Roboto-Regular.ttf");
        for (ResourceLocation loc : new ResourceLocation[] { montserrat, roboto }) {
            try (InputStream in = Minecraft.getMinecraft().getResourceManager().getResource(loc).getInputStream()) {
                return Font.createFont(Font.TRUETYPE_FONT, in).deriveFont(Font.PLAIN, 18f);
            } catch (IOException | FontFormatException ignored) {
            }
        }
        Font sys = new Font("Segoe UI", Font.PLAIN, 18);
        if (sys.getFamily().equals("Dialog")) {
            sys = new Font("SansSerif", Font.PLAIN, 18);
        }
        return sys;
    }

    public static CustomFontRenderer title() {
        ensure();
        return title;
    }

    public static CustomFontRenderer regular() {
        ensure();
        return regular;
    }

    public static CustomFontRenderer small() {
        ensure();
        return small;
    }

    private static void ensure() {
        if (!initialized) {
            init();
        }
    }
}
