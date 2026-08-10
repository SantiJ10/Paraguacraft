package com.paraguacraft.pvp.gui;

import net.minecraft.client.gui.Gui;

/**
 * Sin override de fondos: menús usan dirt/options_background vanilla del pack o del JAR.
 * Los overrides previos (DynamicTexture negro / degradado + tile oscuro) se veían peores
 * y tapaban el look clásico 1.8.
 */
public class GuiBackgroundHandler extends Gui {
    // Sin listeners: se mantiene la clase registrada por compatibilidad de wiring.
}
