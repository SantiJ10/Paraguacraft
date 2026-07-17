package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.gui.theme.MenuTheme;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

/** Selector de tema del menú (estilo Lunar). */
public class ThemeSelectScreen extends Screen {

    private final Screen parent;

    public ThemeSelectScreen(Screen parent) {
        super(Text.literal("Elegir tema"));
        this.parent = parent;
    }

    @Override
    protected void init() {
        int btnW = 180;
        int btnH = 24;
        int cols = 2;
        int startX = width / 2 - (cols * btnW + 8) / 2;
        int startY = 80;
        int gapX = btnW + 8;
        int gapY = 28;
        MenuTheme[] themes = MenuTheme.values();
        for (int i = 0; i < themes.length; i++) {
            MenuTheme theme = themes[i];
            int col = i % cols;
            int row = i / cols;
            int x = startX + col * gapX;
            int y = startY + row * gapY;
            String label = theme.label + (theme == MenuTheme.current() ? " ✓" : "");
            addDrawableChild(ButtonWidget.builder(Text.literal(label), b -> {
                MenuTheme.setCurrent(theme);
                ModernConfig.menuTheme = theme.name();
                ModernConfig.save();
                client.setScreen(new CustomTitleScreen());
            }).dimensions(x, y, btnW, btnH).build());
        }
        addDrawableChild(ButtonWidget.builder(Text.literal("Volver"), b ->
            client.setScreen(parent)).dimensions(width / 2 - btnW / 2, height - 48, btnW, btnH).build());
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        MenuBackground.draw(this, context, mouseX, mouseY, delta);
        context.drawCenteredTextWithShadow(
            textRenderer,
            Text.literal("Tema activo: " + MenuTheme.current().label),
            width / 2,
            52,
            UiTheme.accent()
        );
        super.render(context, mouseX, mouseY, delta);
    }
}
