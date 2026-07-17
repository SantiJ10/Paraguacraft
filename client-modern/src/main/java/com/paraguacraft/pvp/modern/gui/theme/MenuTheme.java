package com.paraguacraft.pvp.modern.gui.theme;

/** Temas del menú — estilo Paraguacraft clásico o Lunar. */
public enum MenuTheme {
    CLASSIC("Clásico", 0x00E5FF, 0x080A12, 0x020308, 0x8899AA),
    LUNAR("Lunar", 0xFFFFFF, 0x121418, 0x08090C, 0xAABBCC),
    JAPAN("Japan", 0xFF6B8A, 0x1A0E14, 0x0A0608, 0xCC8899),
    SUMMER("Verano", 0xFFD166, 0x14120A, 0x080704, 0xBBAA88),
    VANILLA("Vanilla", 0x55FF55, 0x101810, 0x060806, 0x99AA99);

    public final String label;
    public final int accent;
    public final int bgTop;
    public final int bgBottom;
    public final int textDim;

    private static MenuTheme current = CLASSIC;

    MenuTheme(String label, int accent, int bgTop, int bgBottom, int textDim) {
        this.label = label;
        this.accent = accent | 0xFF000000;
        this.bgTop = bgTop | 0xFF000000;
        this.bgBottom = bgBottom | 0xFF000000;
        this.textDim = textDim | 0xFF000000;
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
