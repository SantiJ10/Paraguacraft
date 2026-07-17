package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.narration.NarrationMessageBuilder;
import net.minecraft.client.gui.widget.PressableWidget;
import net.minecraft.client.input.AbstractInput;
import net.minecraft.text.Text;

/** Botón plano estilo Lunar (sin textura de piedra vanilla). */
public class FlatMenuButton extends PressableWidget {

    private final Text message;
    private final Runnable action;

    public FlatMenuButton(int x, int y, int width, int height, Text message, Runnable action) {
        super(x, y, width, height, message);
        this.message = message;
        this.action = action;
    }

    public static FlatMenuButton create(int x, int y, int width, int height, Text message, Runnable action) {
        return new FlatMenuButton(x, y, width, height, message, action);
    }

    @Override
    public void onPress(AbstractInput input) {
        action.run();
    }

    @Override
    protected void appendClickableNarrations(NarrationMessageBuilder builder) {
        appendDefaultNarrations(builder);
    }

    @Override
    protected void drawIcon(DrawContext context, int mouseX, int mouseY, float delta) {
        int bg = isHovered() ? UiTheme.BTN_HOVER : UiTheme.BTN_BG;
        context.fill(getX(), getY(), getX() + getWidth(), getY() + getHeight(), bg);
        if (isHovered()) {
            context.fill(getX(), getY(), getX() + getWidth(), getY() + 1, UiTheme.accent());
        }
        context.drawCenteredTextWithShadow(
            MinecraftClient.getInstance().textRenderer,
            message,
            getX() + getWidth() / 2,
            getY() + (getHeight() - 8) / 2,
            isHovered() ? UiTheme.accent() : UiTheme.TEXT
        );
    }
}
