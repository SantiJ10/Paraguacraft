package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.narration.NarrationMessageBuilder;
import net.minecraft.client.gui.widget.PressableWidget;
import net.minecraft.client.input.AbstractInput;
import net.minecraft.text.Text;

/** Tarjeta estilo Mod Menu 1.8.9 (negro + borde azul al hover). */
public class MenuCardButton extends PressableWidget {

    private final Text title;
    private final Text subtitle;
    private final Runnable action;
    private float hoverAnim;

    public MenuCardButton(int x, int y, int width, int height, Text title, Text subtitle, Runnable action) {
        super(x, y, width, height, title);
        this.title = title;
        this.subtitle = subtitle;
        this.action = action;
    }

    public static MenuCardButton create(int x, int y, int w, int h, String label, boolean on, Runnable action) {
        Text sub = Text.literal(on ? "ON" : "OFF");
        return new MenuCardButton(x, y, w, h, Text.literal(label), sub, action);
    }

    public static MenuCardButton open(int x, int y, int w, int h, String label, Runnable action) {
        return new MenuCardButton(x, y, w, h, Text.literal(label), Text.literal("ABRIR"), action);
    }

    public void setSubtitle(Text sub) {
        // used after toggle refresh via recreate
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
    protected void drawIcon(DrawContext ctx, int mouseX, int mouseY, float delta) {
        float target = isHovered() ? 1f : 0f;
        hoverAnim += (target - hoverAnim) * 0.22f;
        float ease = hoverAnim * hoverAnim * (3f - 2f * hoverAnim);

        int bg = lerpColor(0xE0101218, 0xE8182030, ease);
        ctx.fill(getX(), getY(), getX() + getWidth(), getY() + getHeight(), bg);
        int border = lerpColor(0x33000000, UiTheme.accent(), ease);
        ctx.fill(getX(), getY(), getX() + getWidth(), getY() + 1, border);
        ctx.fill(getX(), getY() + getHeight() - 1, getX() + getWidth(), getY() + getHeight(), border);
        ctx.fill(getX(), getY(), getX() + 1, getY() + getHeight(), border);
        ctx.fill(getX() + getWidth() - 1, getY(), getX() + getWidth(), getY() + getHeight(), border);

        var tr = MinecraftClient.getInstance().textRenderer;
        int tx = getX() + 8;
        ctx.drawText(tr, title, tx, getY() + 8, UiTheme.TEXT, true);
        if (subtitle != null) {
            int subColor = subtitle.getString().equals("ON") ? 0xFF55FF88
                : subtitle.getString().equals("OFF") ? 0xFFFF6666
                : UiTheme.textDim();
            ctx.drawText(tr, subtitle, tx, getY() + getHeight() - 16, subColor, true);
        }
    }

    private static int lerpColor(int a, int b, float t) {
        int aa = (a >> 24) & 0xFF, ar = (a >> 16) & 0xFF, ag = (a >> 8) & 0xFF, ab = a & 0xFF;
        int ba = (b >> 24) & 0xFF, br = (b >> 16) & 0xFF, bg = (b >> 8) & 0xFF, bb = b & 0xFF;
        return ((int) (aa + (ba - aa) * t) << 24)
            | ((int) (ar + (br - ar) * t) << 16)
            | ((int) (ag + (bg - ag) * t) << 8)
            | (int) (ab + (bb - ab) * t);
    }
}
