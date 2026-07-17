package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.ParaguacraftPvPModern;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.TrainingWorldHelper;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import com.paraguacraft.pvp.modern.util.FabricSettingsHelper;
import com.paraguacraft.pvp.modern.util.SkinHelper;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.option.OptionsScreen;
import net.minecraft.client.gui.screen.world.SelectWorldScreen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

/** Menú principal estilo Lunar / Paraguacraft 1.8.9. */
public class CustomTitleScreen extends Screen {

    public CustomTitleScreen() {
        super(Text.literal("Paraguacraft PvP"));
    }

    @Override
    protected void init() {
        int btnW = 220;
        int btnH = 26;
        int startY = height / 2 + 4;
        int gap = 26;
        int row = 0;

        addDrawableChild(ButtonWidget.builder(Text.literal("Un jugador"), b ->
            client.setScreen(new SelectWorldScreen(this))).dimensions(width / 2 - btnW / 2, startY + gap * row++, btnW, btnH).build());
        addDrawableChild(ButtonWidget.builder(Text.literal("Multijugador"), b ->
            client.setScreen(new ParaguacraftMultiplayerScreen(this))).dimensions(width / 2 - btnW / 2, startY + gap * row++, btnW, btnH).build());
        addDrawableChild(ButtonWidget.builder(Text.literal("Hypixel Quick Play"), b ->
            client.setScreen(new HypixelQuickPlayScreen(this))).dimensions(width / 2 - btnW / 2, startY + gap * row++, btnW, btnH).build());
        addDrawableChild(ButtonWidget.builder(Text.literal("Practica PvP (flat)"), b ->
            TrainingWorldHelper.openTrainingWorld(client)).dimensions(width / 2 - btnW / 2, startY + gap * row++, btnW, btnH).build());
        addDrawableChild(ButtonWidget.builder(Text.literal("Mod Menu"), b ->
            client.setScreen(new ModMenuScreen(this))).dimensions(width / 2 - btnW / 2, startY + gap * row++, btnW, btnH).build());
        addDrawableChild(ButtonWidget.builder(Text.literal("Opciones"), b ->
            client.setScreen(new OptionsScreen(this, client.options))).dimensions(width / 2 - btnW / 2, startY + gap * row++, btnW, btnH).build());
        addDrawableChild(ButtonWidget.builder(Text.literal("Salir"), b ->
            client.scheduleStop()).dimensions(width / 2 - btnW / 2, startY + gap * row, btnW, btnH).build());

        int barY = height - 42;
        int iconW = 72;
        int barX = width / 2 - iconW * 2;
        addDrawableChild(ButtonWidget.builder(Text.literal("Skin"), b ->
            SkinHelper.open(this, client)).dimensions(barX, barY, iconW, 22).build());
        addDrawableChild(ButtonWidget.builder(Text.literal("Tema"), b ->
            client.setScreen(new ThemeSelectScreen(this))).dimensions(barX + iconW + 4, barY, iconW, 22).build());
        addDrawableChild(ButtonWidget.builder(Text.literal("Packs"), b ->
            client.setScreen(new PackSelectScreen(this))).dimensions(barX + (iconW + 4) * 2, barY, iconW, 22).build());
        addDrawableChild(ButtonWidget.builder(Text.literal("Fabric"), b ->
            FabricSettingsHelper.open(this, client)).dimensions(barX + (iconW + 4) * 3, barY, iconW, 22).build());
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        MenuBackground.draw(this, context, mouseX, mouseY, delta);
        super.render(context, mouseX, mouseY, delta);
        context.drawText(
            textRenderer,
            Text.literal("Paraguacraft PvP Modern " + ParaguacraftPvPModern.VERSION),
            8,
            height - 12,
            UiTheme.textDim(),
            true
        );
    }
}
