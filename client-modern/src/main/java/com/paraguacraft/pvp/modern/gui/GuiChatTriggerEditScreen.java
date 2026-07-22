package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.core.ChatTriggerConfig;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.TextFieldWidget;
import net.minecraft.text.Text;

/** Editor in-game de keywords para una regla de chat trigger. */
public class GuiChatTriggerEditScreen extends ParaguacraftScreen {

    private final ChatTriggerConfig.Rule rule;
    private TextFieldWidget keywordsField;
    private TextFieldWidget titleField;

    public GuiChatTriggerEditScreen(Screen parent, ChatTriggerConfig.Rule rule) {
        super(Text.literal("Editar trigger"), parent);
        this.rule = rule;
    }

    @Override
    protected void init() {
        int fieldW = 280;
        int cx = width / 2;
        int y = 72;

        titleField = new TextFieldWidget(textRenderer, cx - fieldW / 2, y, fieldW, 20, Text.literal("Titulo"));
        titleField.setMaxLength(48);
        titleField.setText(rule.title == null ? "" : rule.title);
        addSelectableChild(titleField);

        keywordsField = new TextFieldWidget(textRenderer, cx - fieldW / 2, y + 34, fieldW, 20, Text.literal("Keywords"));
        keywordsField.setMaxLength(256);
        keywordsField.setText(rule.keywords == null ? "" : rule.keywords);
        addSelectableChild(keywordsField);

        int btnW = fieldW;
        int btnH = 20;
        addDrawableChild(FlatMenuButton.create(cx - btnW / 2, y + 72, btnW, btnH,
            Text.literal("Guardar"), this::saveAndBack));
        addDrawableChild(FlatMenuButton.create(cx - btnW / 2, y + 98, btnW, btnH,
            Text.literal("Cancelar"), () -> client.setScreen(parent)));
    }

    private void saveAndBack() {
        rule.title = titleField.getText().trim();
        rule.keywords = keywordsField.getText().trim();
        ChatTriggerConfig.save();
        client.setScreen(parent);
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        super.render(context, mouseX, mouseY, delta);
        context.drawCenteredTextWithShadow(textRenderer, Text.literal("Editar chat trigger"), width / 2, 36, UiTheme.accent());
        context.drawTextWithShadow(textRenderer, Text.literal("Titulo en pantalla"), width / 2 - 140, 60, UiTheme.textDim());
        context.drawTextWithShadow(textRenderer, Text.literal("Keywords (coma)"), width / 2 - 140, 94, UiTheme.textDim());
        titleField.render(context, mouseX, mouseY, delta);
        keywordsField.render(context, mouseX, mouseY, delta);
    }
}
