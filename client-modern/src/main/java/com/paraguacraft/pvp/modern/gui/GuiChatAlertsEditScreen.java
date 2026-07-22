package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.core.ChatAlerts;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.TextFieldWidget;
import net.minecraft.text.Text;

/** Editor in-game de palabras clave para chat alerts. */
public class GuiChatAlertsEditScreen extends ParaguacraftScreen {

    private TextFieldWidget wordsField;
    private TextFieldWidget colorField;

    public GuiChatAlertsEditScreen(Screen parent) {
        super(Text.literal("Editar chat alerts"), parent);
    }

    @Override
    protected void init() {
        ChatAlerts.ensureLoaded();
        int fieldW = 280;
        int cx = width / 2;
        int y = 72;

        wordsField = new TextFieldWidget(textRenderer, cx - fieldW / 2, y, fieldW, 20, Text.literal("Palabras"));
        wordsField.setMaxLength(512);
        wordsField.setText(ChatAlerts.wordsCsv());
        addSelectableChild(wordsField);

        colorField = new TextFieldWidget(textRenderer, cx - fieldW / 2, y + 34, fieldW, 20, Text.literal("Color"));
        colorField.setMaxLength(24);
        colorField.setText(ChatAlerts.color == null ? "YELLOW" : ChatAlerts.color);
        addSelectableChild(colorField);

        int btnW = fieldW;
        addDrawableChild(FlatMenuButton.create(cx - btnW / 2, y + 72, btnW, 20,
            Text.literal("Guardar"), this::saveAndBack));
        addDrawableChild(FlatMenuButton.create(cx - btnW / 2, y + 98, btnW, 20,
            Text.literal("Cancelar"), () -> client.setScreen(parent)));
    }

    private void saveAndBack() {
        ChatAlerts.setWordsFromCsv(wordsField.getText());
        ChatAlerts.color = colorField.getText().trim().isEmpty() ? "YELLOW" : colorField.getText().trim();
        ChatAlerts.save();
        client.setScreen(parent);
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        super.render(context, mouseX, mouseY, delta);
        context.drawCenteredTextWithShadow(textRenderer, Text.literal("Editar chat alerts"), width / 2, 36, UiTheme.accent());
        context.drawTextWithShadow(textRenderer, Text.literal("Palabras (coma)"), width / 2 - 140, 60, UiTheme.textDim());
        context.drawTextWithShadow(textRenderer, Text.literal("Color (YELLOW, RED...)"), width / 2 - 140, 94, UiTheme.textDim());
        wordsField.render(context, mouseX, mouseY, delta);
        colorField.render(context, mouseX, mouseY, delta);
    }
}
