package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.GameMenuScreen;

/** Menú de pausa con overlay oscuro Paraguacraft. */
public class CustomPauseScreen extends GameMenuScreen {

    public CustomPauseScreen(boolean showMenu) {
        super(showMenu);
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        context.fill(0, 0, width, height, UiTheme.OVERLAY);
        super.render(context, mouseX, mouseY, delta);
    }
}
