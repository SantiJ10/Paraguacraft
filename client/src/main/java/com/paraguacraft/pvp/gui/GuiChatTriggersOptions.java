package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.core.ModLang;
import com.paraguacraft.pvp.gui.theme.UiTheme;
import com.paraguacraft.pvp.modules.ChatTriggerConfig;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiButton;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.gui.GuiTextField;
import org.lwjgl.input.Keyboard;

import java.io.IOException;
import java.util.List;

/** Editor de reglas de alertas en chat. */
public class GuiChatTriggersOptions extends GuiScreen {

    private List<ChatTriggerConfig.Rule> working;
    private int selected = -1;
    private GuiTextField keywordsField;
    private GuiTextField titleField;
    private GuiTextField colorField;
    private int scroll;

    @Override
    public void initGui() {
        ChatTriggerConfig.ensureLoaded();
        working = ChatTriggerConfig.cloneRules();
        buttonList.clear();
        keywordsField = new GuiTextField(0, fontRendererObj, width / 2 - 140, height / 2 + 20, 280, 18);
        titleField = new GuiTextField(1, fontRendererObj, width / 2 - 140, height / 2 + 58, 280, 18);
        colorField = new GuiTextField(2, fontRendererObj, width / 2 - 140, height / 2 + 96, 120, 18);
        if (selected >= 0 && selected < working.size()) {
            fillFields(working.get(selected));
        }
        buttonList.add(new GuiButton(1, width / 2 - 150, height - 52, 70, 20, ModLang.format("paraguacraft.triggers.add")));
        buttonList.add(new GuiButton(2, width / 2 - 72, height - 52, 70, 20, ModLang.format("paraguacraft.triggers.remove")));
        buttonList.add(new GuiButton(3, width / 2 + 6, height - 52, 70, 20, ModLang.format("paraguacraft.triggers.reset")));
        buttonList.add(new GuiButton(4, width / 2 + 84, height - 52, 70, 20, ModLang.format("paraguacraft.triggers.save")));
    }

    private void fillFields(ChatTriggerConfig.Rule rule) {
        keywordsField.setText(rule.keywords);
        titleField.setText(rule.title);
        colorField.setText(rule.color);
    }

    private void applyFields(ChatTriggerConfig.Rule rule) {
        rule.keywords = keywordsField.getText().trim();
        rule.title = titleField.getText().trim();
        rule.color = colorField.getText().trim().isEmpty() ? "RED" : colorField.getText().trim().toUpperCase();
    }

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        drawRect(0, 0, width, height, 0x99000000);
        int px = width / 2 - 160;
        int py = height / 2 - 120;
        Gui.drawRect(px, py, px + 320, py + 260, 0xCC0A0C14);
        FontRenderer fr = fontRendererObj;
        fr.drawStringWithShadow(ModLang.format("paraguacraft.triggers.title"), px + 12, py + 10, UiTheme.ACCENT);
        fr.drawStringWithShadow(ModLang.format("paraguacraft.triggers.hint"), px + 12, py + 24, UiTheme.TEXT_DIM);

        int listY = py + 42;
        for (int i = 0; i < working.size(); i++) {
            ChatTriggerConfig.Rule rule = working.get(i);
            int rowY = listY + i * 18 - scroll;
            if (rowY < py + 38 || rowY > py + 118) {
                continue;
            }
            boolean sel = i == selected;
            boolean hover = mouseX >= px + 8 && mouseX <= px + 312 && mouseY >= rowY && mouseY <= rowY + 16;
            Gui.drawRect(px + 8, rowY, px + 312, rowY + 16, sel ? 0x4400E5FF : (hover ? 0x33223344 : 0x22000000));
            String label = (rule.enabled ? "" : "[OFF] ") + rule.title;
            fr.drawStringWithShadow(label, px + 14, rowY + 4, rule.enabled ? UiTheme.TEXT : UiTheme.TEXT_DIM);
        }

        if (selected >= 0 && selected < working.size()) {
            fr.drawStringWithShadow(ModLang.format("paraguacraft.triggers.keywords"), px + 12, height / 2 + 8, UiTheme.TEXT_DIM);
            keywordsField.drawTextBox();
            fr.drawStringWithShadow(ModLang.format("paraguacraft.triggers.alert_title"), px + 12, height / 2 + 46, UiTheme.TEXT_DIM);
            titleField.drawTextBox();
            fr.drawStringWithShadow(ModLang.format("paraguacraft.triggers.color"), px + 12, height / 2 + 84, UiTheme.TEXT_DIM);
            colorField.drawTextBox();
        }

        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    @Override
    protected void mouseClicked(int mouseX, int mouseY, int mouseButton) throws IOException {
        keywordsField.mouseClicked(mouseX, mouseY, mouseButton);
        titleField.mouseClicked(mouseX, mouseY, mouseButton);
        colorField.mouseClicked(mouseX, mouseY, mouseButton);
        int px = width / 2 - 160;
        int py = height / 2 - 120;
        int listY = py + 42;
        for (int i = 0; i < working.size(); i++) {
            int rowY = listY + i * 18 - scroll;
            if (mouseX >= px + 8 && mouseX <= px + 312 && mouseY >= rowY && mouseY <= rowY + 16) {
                selected = i;
                fillFields(working.get(i));
                return;
            }
        }
        super.mouseClicked(mouseX, mouseY, mouseButton);
    }

    @Override
    protected void keyTyped(char typedChar, int keyCode) throws IOException {
        if (keyCode == Keyboard.KEY_ESCAPE) {
            mc.displayGuiScreen(new GuiParaguaMenu());
            return;
        }
        if (keywordsField.textboxKeyTyped(typedChar, keyCode)) {
            return;
        }
        if (titleField.textboxKeyTyped(typedChar, keyCode)) {
            return;
        }
        if (colorField.textboxKeyTyped(typedChar, keyCode)) {
            return;
        }
        super.keyTyped(typedChar, keyCode);
    }

    @Override
    protected void actionPerformed(GuiButton button) throws IOException {
        if (button.id == 1) {
            ChatTriggerConfig.Rule rule = new ChatTriggerConfig.Rule("custom", "texto", "ALERTA", "YELLOW");
            working.add(rule);
            selected = working.size() - 1;
            fillFields(rule);
        } else if (button.id == 2 && selected >= 0 && selected < working.size()) {
            working.remove(selected);
            selected = Math.min(selected, working.size() - 1);
            if (selected >= 0) {
                fillFields(working.get(selected));
            }
        } else if (button.id == 3) {
            working = ChatTriggerConfig.cloneRules();
            ChatTriggerConfig.resetDefaults();
            working = ChatTriggerConfig.cloneRules();
            selected = 0;
            fillFields(working.get(0));
        } else if (button.id == 4) {
            if (selected >= 0 && selected < working.size()) {
                applyFields(working.get(selected));
            }
            ChatTriggerConfig.setRules(working);
            ModConfig.chatTriggers = true;
            ModConfig.save();
            mc.displayGuiScreen(new GuiParaguaMenu());
        }
    }

    @Override
    public void updateScreen() {
        keywordsField.updateCursorCounter();
        titleField.updateCursorCounter();
        colorField.updateCursorCounter();
    }

    @Override
    public boolean doesGuiPauseGame() {
        return false;
    }
}
