package com.paraguacraft.pvp.modern.gui.theme;

import com.paraguacraft.pvp.modern.ParaguacraftPvPModern;
import net.minecraft.util.Identifier;

/** Temas del menu con fondo completo (gradiente, imagen o animacion). */
public enum MenuTheme {
    CLASSIC("Clasico", 0x00E5FF, 0x080A12, 0x020308, 0x8899AA, null, 0x00000000, 0x00000000, true),
    LUNAR("Lunar", 0xFFFFFF, 0x121418, 0x08090C, 0xAABBCC, "textures/gui/bg_lunar.png", 0x66000000, 0x00000000, true),
    JAPAN("Japan", 0xFF6B8A, 0x1A0E14, 0x0A0608, 0xCC8899, "textures/gui/bg_japan.png", 0x77001018, 0x44FF2244, false),
    SUMMER("Verano", 0xFFD166, 0x14120A, 0x080704, 0xBBAA88, "textures/gui/bg_summer.png", 0x66141008, 0x00000000, true),
    VANILLA("Vanilla", 0x55FF55, 0x101810, 0x060806, 0x99AA99, "textures/gui/bg_vanilla.png", 0x77000808, 0x00000000, false);

    public final String label;
    public final int accent;
    public final int bgTop;
    public final int bgBottom;
    public final int textDim;
    public final Identifier backgroundTexture;
    public final int backgroundTint;
    public final int secondaryTint;
    public final boolean animatedOverlay;

    private static MenuTheme current = CLASSIC;

    MenuTheme(String label, int accent, int bgTop, int bgBottom, int textDim,
              String texturePath, int backgroundTint, int secondaryTint, boolean animatedOverlay) {
        this.label = label;
        this.accent = accent | 0xFF000000;
        this.bgTop = bgTop | 0xFF000000;
        this.bgBottom = bgBottom | 0xFF000000;
        this.textDim = textDim | 0xFF000000;
        this.backgroundTexture = texturePath == null
            ? null
            : Identifier.of(ParaguacraftPvPModern.MOD_ID, texturePath);
        this.backgroundTint = backgroundTint;
        this.secondaryTint = secondaryTint;
        this.animatedOverlay = animatedOverlay;
    }

    public static MenuTheme current() {
        return current;
    }

    public static void setCurrent(MenuTheme theme) {
        if (theme != null) {
            current = theme;
        }
    }

    public static MenuTheme fromName(String name) {
        if (name == null) {
            return CLASSIC;
        }
        for (MenuTheme t : values()) {
            if (t.name().equalsIgnoreCase(name.trim())) {
                return t;
            }
        }
        return CLASSIC;
    }

    public MenuTheme next() {
        MenuTheme[] all = values();
        return all[(ordinal() + 1) % all.length];
    }
}
