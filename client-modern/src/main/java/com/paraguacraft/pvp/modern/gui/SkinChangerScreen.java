package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.SkinManager;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.client.gui.widget.TextFieldWidget;
import net.minecraft.text.Text;

/** Skin Changer estilo Lunar: URL o nombre de jugador. */
public class SkinChangerScreen extends Screen {

    private final Screen parent;
    private TextFieldWidget urlField;

    public SkinChangerScreen(Screen parent) {
        super(Text.literal("Skin Changer"));
        this.parent = parent;
    }

    @Override
    protected void init() {
        int panelW = Math.min(360, width - 40);
        int cx = width / 2;
        int y = height / 2 - 60;

        urlField = new TextFieldWidget(textRenderer, cx - panelW / 2, y, panelW, 20, Text.literal("Skin URL o nick"));
        urlField.setMaxLength(256);
        urlField.setText(ModernConfig.customSkinUrl == null ? "" : ModernConfig.customSkinUrl);
        urlField.setPlaceholder(Text.literal("URL .png o nombre de Minecraft"));
        addSelectableChild(urlField);
        setInitialFocus(urlField);

        addDrawableChild(ButtonWidget.builder(Text.literal("Aplicar skin"), b -> {
            ModernConfig.customSkinUrl = urlField.getText().trim();
            SkinManager.apply(ModernConfig.customSkinUrl);
            ModernConfig.save();
        }).dimensions(cx - panelW / 2, y + 28, panelW, 22).build());

        addDrawableChild(ButtonWidget.builder(Text.literal("Quitar skin custom"), b -> {
            ModernConfig.customSkinUrl = "";
            SkinManager.clear();
            urlField.setText("");
            ModernConfig.save();
        }).dimensions(cx - panelW / 2, y + 56, panelW, 22).build());

        addDrawableChild(ButtonWidget.builder(Text.literal("Volver"), b ->
            client.setScreen(parent)).dimensions(cx - panelW / 2, y + 88, panelW, 22).build());
    }

    @Override
    public void render(DrawContext ctx, int mouseX, int mouseY, float delta) {
        MenuBackground.draw(this, ctx, mouseX, mouseY, delta);
        int panelW = Math.min(360, width - 40);
        int cx = width / 2;
        int y = height / 2 - 60;
        ctx.fill(cx - panelW / 2 - 8, y - 28, cx + panelW / 2 + 8, y + 120, 0xCC101218);
        ctx.drawCenteredTextWithShadow(textRenderer, Text.literal("Skin Changer"), cx, y - 20, UiTheme.accent());
        ctx.drawCenteredTextWithShadow(textRenderer, Text.literal("Pega URL de skin o nick para copiar su skin"), cx, y - 8, UiTheme.textDim());
        urlField.render(ctx, mouseX, mouseY, delta);
        super.render(ctx, mouseX, mouseY, delta);
    }
}
