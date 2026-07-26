package com.paraguacraft.pvp.modern.gui.theme;

public final class UiTheme {

    /** ~55% negro de fondo de menu. */
    public static final int OVERLAY = 0x8C000000;
    /** Boton negro ~60% opacidad. */
    public static final int BTN_BG = 0x99000000;
    public static final int BTN_HOVER = 0xB0000000;
    /** ARGB completo: en 1.21 drawText ignora colores con alpha 0. */
    public static final int TEXT = 0xFFF0F4FF;
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
