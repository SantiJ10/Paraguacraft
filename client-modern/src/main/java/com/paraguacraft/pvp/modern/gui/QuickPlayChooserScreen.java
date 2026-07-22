package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;

/** Elige servidor para Quick Play (`). */
public class QuickPlayChooserScreen extends ParaguacraftScreen {

    public QuickPlayChooserScreen(Screen parent) {
        super(Text.literal("Quick Play"), parent);
    }

    @Override
    protected void init() {
        int btnW = 240;
        int btnH = 22;
        int y = height / 2 - 30;
        addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, y, btnW, btnH,
            Text.literal("Hypixel"), () -> client.setScreen(new HypixelQuickPlayScreen(parent != null ? parent : this))));
        addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, y + 28, btnW, btnH,
            Text.literal("Cubecraft"), () -> client.setScreen(new CubecraftQuickPlayScreen(parent != null ? parent : this))));
        addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, y + 60, btnW, btnH,
            Text.literal("Volver"), () -> client.setScreen(parent)));
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        super.render(context, mouseX, mouseY, delta);
        context.drawCenteredTextWithShadow(textRenderer, Text.literal("Quick Play"), width / 2, yTitle(), UiTheme.accent());
        context.drawCenteredTextWithShadow(textRenderer, Text.literal("Elegi servidor"), width / 2, yTitle() + 14, UiTheme.textDim());
    }

    private int yTitle() {
        return height / 2 - 58;
    }
}
