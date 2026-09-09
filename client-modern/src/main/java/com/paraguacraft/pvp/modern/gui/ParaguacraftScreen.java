package com.paraguacraft.pvp.modern.gui;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.TitleScreen;
import net.minecraft.text.Text;

/** Pantalla Paraguacraft: overlay sobre el mundo in-game, constelación solo en el menu. */
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
    public boolean shouldPause() {
        return false;
    }

    @Override
    public void renderBackground(DrawContext context, int mouseX, int mouseY, float delta) {
        if (client != null && client.world != null) {
            context.fill(0, 0, width, height, 0x66000000);
            return;
        }
        MenuBackground.draw(this, context, mouseX, mouseY, delta, false);
    }
}
