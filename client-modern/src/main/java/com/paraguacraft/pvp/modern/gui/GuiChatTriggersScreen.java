package com.paraguacraft.pvp.modern.gui;



import com.paraguacraft.pvp.modern.core.ChatTriggerConfig;

import com.paraguacraft.pvp.modern.gui.theme.UiTheme;

import net.minecraft.client.gui.DrawContext;

import net.minecraft.client.gui.screen.Screen;

import net.minecraft.text.Text;



/** Lista de reglas de chat triggers (titulos en pantalla). */

public class GuiChatTriggersScreen extends ParaguacraftScreen {



    public GuiChatTriggersScreen(Screen parent) {

        super(Text.literal("Chat triggers"), parent);

    }



    @Override

    protected void init() {

        ChatTriggerConfig.ensureLoaded();

        int btnW = 280;

        int btnH = 20;

        int startY = 58;

        int gap = 22;

        int i = 0;

        for (ChatTriggerConfig.Rule rule : ChatTriggerConfig.getRules()) {

            if (rule == null) {

                continue;

            }

            int y = startY + i * gap;

            String label = (rule.enabled ? "ON" : "OFF") + " · " + rule.title;

            ChatTriggerConfig.Rule ref = rule;

            addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, y, btnW, btnH,

                Text.literal(label), () -> client.setScreen(new GuiChatTriggerEditScreen(this, ref))));

            i++;

        }

        int after = startY + i * gap + 6;

        addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, after, btnW, btnH,

            Text.literal("Restaurar reglas default"), () -> {

                ChatTriggerConfig.resetDefaults();

                rebuild();

            }));

        addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, after + 26, btnW, btnH,

            Text.literal("Volver"), () -> client.setScreen(parent)));

    }



    private void rebuild() {

        clearChildren();

        init();

    }



    @Override

    public void render(DrawContext context, int mouseX, int mouseY, float delta) {

        super.render(context, mouseX, mouseY, delta);

        context.drawCenteredTextWithShadow(textRenderer, Text.literal("Chat triggers"), width / 2, 36, UiTheme.accent());

        context.drawCenteredTextWithShadow(

            textRenderer,

            Text.literal("Click en una regla para editar titulo y keywords"),

            width / 2,

            48,

            UiTheme.textDim()

        );

    }

}


