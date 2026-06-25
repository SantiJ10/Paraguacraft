package com.paraguacraft.pvp.hud;

import com.paraguacraft.pvp.gui.theme.UiTheme;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.FontRenderer;

/** Helpers de dibujo HUD — fuente vanilla (estable con OptiFine). */
public final class HudDraw {

    private static final Minecraft MC = Minecraft.getMinecraft();

    private HudDraw() {}

    private static FontRenderer font() {
        return MC.fontRendererObj;
    }

    public static void labeled(String label, String value, float x, float y) {
        FontRenderer fr = font();
        fr.drawStringWithShadow(label, (int) x, (int) y, UiTheme.ACCENT);
        fr.drawStringWithShadow(value, (int) (x + fr.getStringWidth(label)), (int) y, UiTheme.TEXT);
    }

    public static void text(String line, float x, float y, int color) {
        font().drawStringWithShadow(line, (int) x, (int) y, color);
    }

    public static void centered(String line, float cx, float y, int color) {
        FontRenderer fr = font();
        fr.drawStringWithShadow(line, (int) (cx - fr.getStringWidth(line) / 2f), (int) y, color);
    }

    public static int width(String line) {
        return font().getStringWidth(line);
    }
}
