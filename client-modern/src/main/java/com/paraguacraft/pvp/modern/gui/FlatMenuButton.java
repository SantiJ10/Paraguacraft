package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.narration.NarrationMessageBuilder;
import net.minecraft.client.gui.widget.PressableWidget;
import net.minecraft.client.input.AbstractInput;
import net.minecraft.text.Text;

/** Boton estilo 1.8.9: negro con borde azul al hover. */
public class FlatMenuButton extends PressableWidget {

    private final Text message;
    private final Runnable action;
    private float hoverAnim;

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
        float target = isHovered() ? 1f : 0f;
        hoverAnim += (target - hoverAnim) * 0.22f;
        float ease = hoverAnim * hoverAnim * (3f - 2f * hoverAnim);

        int bg = lerpColor(UiTheme.BTN_BG, UiTheme.BTN_HOVER, ease);
        context.fill(getX(), getY(), getX() + getWidth(), getY() + getHeight(), bg);

        if (ease > 0.02f) {
            int border = lerpColor(0x00000000, UiTheme.accent(), ease);
            context.fill(getX(), getY(), getX() + getWidth(), getY() + 1, border);
            context.fill(getX(), getY() + getHeight() - 1, getX() + getWidth(), getY() + getHeight(), border);
            context.fill(getX(), getY(), getX() + 1, getY() + getHeight(), border);
            context.fill(getX() + getWidth() - 1, getY(), getX() + getWidth(), getY() + getHeight(), border);
        }

        int textColor = lerpColor(UiTheme.TEXT, UiTheme.accent(), ease);
        context.drawCenteredTextWithShadow(
            MinecraftClient.getInstance().textRenderer,
            message,
            getX() + getWidth() / 2,
            getY() + (getHeight() - 8) / 2,
            textColor
        );
    }

    private static int lerpColor(int a, int b, float t) {
        int aa = (a >> 24) & 0xFF, ar = (a >> 16) & 0xFF, ag = (a >> 8) & 0xFF, ab = a & 0xFF;
        int ba = (b >> 24) & 0xFF, br = (b >> 16) & 0xFF, bg = (b >> 8) & 0xFF, bb = b & 0xFF;
        int ca = (int) (aa + (ba - aa) * t);
        int cr = (int) (ar + (br - ar) * t);
        int cg = (int) (ag + (bg - ag) * t);
        int cb = (int) (ab + (bb - ab) * t);
        return (ca << 24) | (cr << 16) | (cg << 8) | cb;
    }
}
