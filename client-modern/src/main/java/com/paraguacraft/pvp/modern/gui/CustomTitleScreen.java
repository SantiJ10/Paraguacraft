package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.ParaguacraftPvPModern;
import com.paraguacraft.pvp.modern.core.TrainingWorldHelper;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import com.paraguacraft.pvp.modern.util.FabricSettingsHelper;
import com.paraguacraft.pvp.modern.util.SkinHelper;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.option.OptionsScreen;
import net.minecraft.client.gui.screen.world.SelectWorldScreen;
import net.minecraft.text.Text;

/** Menú principal estilo Lunar / Paraguacraft 1.8.9. */
public class CustomTitleScreen extends Screen {

    public CustomTitleScreen() {
        super(Text.literal("Paraguacraft PvP"));
    }

    @Override
    protected void init() {
        int btnW = Math.min(200, width - 48);
        int btnH = 20;
        int gap = 22;
        int cx = width / 2 - btnW / 2;

        int startY = MenuBackground.contentTop(height);
        int footerH = 22 + 10 + 22 + 16;
        int mainBlock = 4 * gap + btnH;
        if (startY + mainBlock + footerH > height - 8) {
            startY = height - mainBlock - footerH - 8;
        }

        int row = 0;
        addDrawableChild(FlatMenuButton.create(cx, startY + gap * row++, btnW, btnH, Text.literal("Un jugador"), () ->
            client.setScreen(new SelectWorldScreen(this))));
        addDrawableChild(FlatMenuButton.create(cx, startY + gap * row++, btnW, btnH, Text.literal("Multijugador"), () ->
            client.setScreen(new ParaguacraftMultiplayerScreen(this))));
        addDrawableChild(FlatMenuButton.create(cx, startY + gap * row++, btnW, btnH, Text.literal("Hypixel Quick Play"), () ->
            client.setScreen(new HypixelQuickPlayScreen(this))));
        addDrawableChild(FlatMenuButton.create(cx, startY + gap * row, btnW, btnH, Text.literal("Practica PvP (flat)"), () ->
            TrainingWorldHelper.openTrainingWorld(client)));

        int utilY = height - 74;
        int utilW = Math.min(96, (width - 56) / 3);
        int utilGap = 6;
        int utilTotal = utilW * 3 + utilGap * 2;
        int utilX = width / 2 - utilTotal / 2;
        addDrawableChild(FlatMenuButton.create(utilX, utilY, utilW, btnH, Text.literal("Mod Menu"), () ->
            client.setScreen(new ModMenuScreen(this))));
        addDrawableChild(FlatMenuButton.create(utilX + utilW + utilGap, utilY, utilW, btnH, Text.literal("Opciones"), () ->
            client.setScreen(new OptionsScreen(this, client.options))));
        addDrawableChild(FlatMenuButton.create(utilX + (utilW + utilGap) * 2, utilY, utilW, btnH, Text.literal("Salir"), () ->
            client.scheduleStop()));

        int barY = height - 42;
        int iconW = Math.min(72, (width - 48) / 4);
        int iconGap = 4;
        int barTotal = iconW * 4 + iconGap * 3;
        int barX = width / 2 - barTotal / 2;
        addDrawableChild(FlatMenuButton.create(barX, barY, iconW, btnH, Text.literal("Skin"), () ->
            SkinHelper.open(this, client)));
        addDrawableChild(FlatMenuButton.create(barX + iconW + iconGap, barY, iconW, btnH, Text.literal("Tema"), () ->
            client.setScreen(new ThemeSelectScreen(this))));
        addDrawableChild(FlatMenuButton.create(barX + (iconW + iconGap) * 2, barY, iconW, btnH, Text.literal("Packs"), () ->
            client.setScreen(new PackSelectScreen(this))));
        addDrawableChild(FlatMenuButton.create(barX + (iconW + iconGap) * 3, barY, iconW, btnH, Text.literal("Fabric"), () ->
            FabricSettingsHelper.open(this, client)));
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
