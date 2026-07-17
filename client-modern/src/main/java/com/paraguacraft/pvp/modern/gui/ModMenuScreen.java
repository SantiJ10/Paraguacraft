package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.PerformanceConfig;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

/** Menú del mod estilo Lunar (Right Shift). */
public class ModMenuScreen extends Screen {

    private final Screen parent;

    public ModMenuScreen(Screen parent) {
        super(Text.literal("Paraguacraft PvP"));
        this.parent = parent;
    }

    @Override
    protected void init() {
        int panelW = Math.min(width - 40, 420);
        int panelX = width / 2 - panelW / 2;
        int y = height / 2 - 120;
        int btnW = panelW - 32;
        int btnH = 22;
        int gap = 24;

        addToggle(panelX + 16, y, btnW, btnH, "FPS", () -> ModernConfig.showFps, v -> ModernConfig.showFps = v);
        y += gap;
        addToggle(panelX + 16, y, btnW, btnH, "Ping", () -> ModernConfig.showPing, v -> ModernConfig.showPing = v);
        y += gap;
        addToggle(panelX + 16, y, btnW, btnH, "Keystrokes", () -> ModernConfig.showKeystrokes, v -> ModernConfig.showKeystrokes = v);
        y += gap;
        addToggle(panelX + 16, y, btnW, btnH, "Coordenadas", () -> ModernConfig.showCoords, v -> ModernConfig.showCoords = v);
        y += gap;
        addToggle(panelX + 16, y, btnW, btnH, "Armadura", () -> ModernConfig.showArmor, v -> ModernConfig.showArmor = v);
        y += gap;
        addToggle(panelX + 16, y, btnW, btnH, "CPS", () -> ModernConfig.showCps, v -> ModernConfig.showCps = v);
        y += gap;
        addToggle(panelX + 16, y, btnW, btnH, "Boost FPS", () -> PerformanceConfig.boostFps, v -> PerformanceConfig.boostFps = v);
        y += gap;
        addToggle(panelX + 16, y, btnW, btnH, "Toggle sprint", () -> ModernConfig.toggleSprint, v -> {
            ModernConfig.toggleSprint = v;
            if (client != null) {
                client.options.getSprintToggled().setValue(v);
            }
        });
        y += gap + 8;

        addDrawableChild(ButtonWidget.builder(Text.literal("Hypixel Quick Play"), b ->
            client.setScreen(new HypixelQuickPlayScreen(this))).dimensions(panelX + 16, y, btnW, btnH).build());
        y += gap;
        addDrawableChild(ButtonWidget.builder(Text.literal("Texture packs"), b ->
            client.setScreen(new PackSelectScreen(this))).dimensions(panelX + 16, y, btnW, btnH).build());
        y += gap;
        addDrawableChild(ButtonWidget.builder(Text.literal("Cerrar"), b -> goBack()).dimensions(panelX + 16, y, btnW, btnH).build());
    }

    private interface BoolGetter {
        boolean get();
    }

    private interface BoolSetter {
        void set(boolean value);
    }

    private void addToggle(int x, int y, int w, int h, String label, BoolGetter getter, BoolSetter setter) {
        addDrawableChild(ButtonWidget.builder(Text.literal(toggleLabel(label, getter.get())), b -> {
            setter.set(!getter.get());
            ModernConfig.save();
            clearChildren();
            init();
        }).dimensions(x, y, w, h).build());
    }

    private static String toggleLabel(String label, boolean on) {
        return label + ": " + (on ? "ON" : "OFF");
    }

    private void goBack() {
        ModernConfig.save();
        if (parent != null) {
            client.setScreen(parent);
        } else {
            client.setScreen(null);
        }
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        context.fill(0, 0, width, height, 0xCC000000);
        int panelW = Math.min(width - 40, 420);
        int panelH = Math.min(height - 40, 340);
        int panelX = width / 2 - panelW / 2;
        int panelY = height / 2 - panelH / 2;
        context.fill(panelX, panelY, panelX + panelW, panelY + panelH, 0xE0101218);
        context.drawCenteredTextWithShadow(textRenderer, Text.literal("Paraguacraft PvP"), width / 2, panelY + 10, UiTheme.accent());
        context.drawCenteredTextWithShadow(textRenderer, Text.literal("Right Shift · toggles"), width / 2, panelY + 24, UiTheme.textDim());
        super.render(context, mouseX, mouseY, delta);
    }
}
