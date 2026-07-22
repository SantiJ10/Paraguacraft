package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.ChatAlerts;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;

/** Opciones de alertas en chat (palabras clave). */
public class GuiChatAlertsScreen extends ParaguacraftScreen {

    public GuiChatAlertsScreen(Screen parent) {
        super(Text.literal("Chat alerts"), parent);
    }

    @Override
    protected void init() {
        ChatAlerts.ensureLoaded();
        int btnW = 260;
        int btnH = 22;
        int y = 64;
        int gap = 26;

        addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, y, btnW, btnH,
            Text.literal("Activo: " + onOff(ModernConfig.chatAlertsEnabled)), () -> {
                ModernConfig.chatAlertsEnabled = !ModernConfig.chatAlertsEnabled;
                ChatAlerts.enabled = ModernConfig.chatAlertsEnabled;
                ChatAlerts.save();
                ModernConfig.save();
                rebuild();
            }));
        addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, y + gap, btnW, btnH,
            Text.literal("Sonido: " + onOff(ChatAlerts.sound)), () -> {
                ChatAlerts.sound = !ChatAlerts.sound;
                ChatAlerts.save();
                rebuild();
            }));
        addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, y + gap * 2, btnW, btnH,
            Text.literal("Resaltar en chat: " + onOff(ChatAlerts.highlight)), () -> {
                ChatAlerts.highlight = !ChatAlerts.highlight;
                ChatAlerts.save();
                rebuild();
            }));
        addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, y + gap * 3, btnW, btnH,
            Text.literal("Restaurar palabras default"), () -> {
                ChatAlerts.resetDefaults();
                rebuild();
            }));
        addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, y + gap * 4 + 8, btnW, btnH,
            Text.literal("Volver"), () -> client.setScreen(parent)));
    }

    private void rebuild() {
        clearChildren();
        init();
    }

    private static String onOff(boolean v) {
        return v ? "ON" : "OFF";
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        super.render(context, mouseX, mouseY, delta);
        context.drawCenteredTextWithShadow(textRenderer, Text.literal("Chat alerts"), width / 2, 40, UiTheme.accent());
        context.drawCenteredTextWithShadow(
            textRenderer,
            Text.literal("Palabras en paraguacraft/chat_alerts.json"),
            width / 2,
            52,
            UiTheme.textDim()
        );
    }
}
