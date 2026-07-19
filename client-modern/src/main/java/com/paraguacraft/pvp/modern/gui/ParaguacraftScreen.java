package com.paraguacraft.pvp.modern.gui;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.TitleScreen;
import net.minecraft.text.Text;

/** Pantalla Paraguacraft con fondo de constelación (sin logo duplicado). */
public abstract class ParaguacraftScreen extends Screen {

    protected final Screen parent;

    protected ParaguacraftScreen(Text title, Screen parent) {
        super(title);
        this.parent = parent;
    }

    @Override
    public void close() {
        if (client == null) {
            return;
        }
        Screen target = parent != null ? parent : new CustomTitleScreen();
        if (target instanceof TitleScreen) {
            target = new CustomTitleScreen();
        }
        client.setScreen(target);
    }

    @Override
    public void renderBackground(DrawContext context, int mouseX, int mouseY, float delta) {
        MenuBackground.draw(this, context, mouseX, mouseY, delta, false);
    }
}
