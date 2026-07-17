package com.paraguacraft.pvp.modern.gui.theme;

public final class UiTheme {

    public static final int OVERLAY = 0x8C000000;
    public static final int BTN_BG = 0xC80C0E16;
    public static final int BTN_HOVER = 0xDC121622;
    public static final int TEXT = 0xF0F4FF;
    public static final int BAR_BG = 0xB0101218;

    private UiTheme() {}

    public static int accent() {
        return MenuTheme.current().accent;
    }

    public static int bgTop() {
        return MenuTheme.current().bgTop;
    }

    public static int bgBottom() {
        return MenuTheme.current().bgBottom;
    }

    public static int textDim() {
        return MenuTheme.current().textDim;
    }
}
