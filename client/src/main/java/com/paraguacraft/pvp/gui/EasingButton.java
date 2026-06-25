package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.gui.theme.UiEasing;
import com.paraguacraft.pvp.gui.theme.UiTheme;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiButton;
import net.minecraft.client.renderer.GlStateManager;

/**
 * Botón minimalista con hover animado (easing) y texto CFont.
 */
public class EasingButton extends GuiButton {

    private float hoverAnim;
    public boolean isToggled;

    public EasingButton(int id, int x, int y, int w, int h, String label) {
        this(id, x, y, w, h, label, false);
    }

    public EasingButton(int id, int x, int y, int w, int h, String label, boolean toggled) {
        super(id, x, y, w, h, label);
        this.isToggled = toggled;
    }

    @Override
    public void drawButton(Minecraft mc, int mouseX, int mouseY) {
        if (!this.visible) {
            return;
        }
        this.hovered = mouseX >= this.xPosition && mouseY >= this.yPosition
            && mouseX < this.xPosition + this.width && mouseY < this.yPosition + this.height;

        float target = (this.hovered || this.isToggled) ? 1f : 0f;
        hoverAnim = UiEasing.approach(hoverAnim, target, 0.18f);
        float ease = UiEasing.easeOutCubic(hoverAnim);

        int bg = lerpColor(UiTheme.BTN_BG, UiTheme.BTN_HOVER, ease);
        if (this.isToggled) {
            bg = lerpColor(bg, UiTheme.ACCENT_DIM, 0.55f);
        }
        Gui.drawRect(this.xPosition, this.yPosition, this.xPosition + this.width, this.yPosition + this.height, bg);

        if (ease > 0.02f || this.isToggled) {
            int border = lerpColor(0x00000000, UiTheme.ACCENT, ease);
            drawBorder(border);
        }

        float scale = 1f + ease * 0.02f;
        GlStateManager.pushMatrix();
        GlStateManager.translate(this.xPosition + this.width / 2f, this.yPosition + this.height / 2f, 0);
        GlStateManager.scale(scale, scale, 1f);
        int textColor = lerpColor(UiTheme.TEXT, UiTheme.ACCENT, ease);
        mc.fontRendererObj.drawStringWithShadow(
            this.displayString,
            -mc.fontRendererObj.getStringWidth(this.displayString) / 2,
            -4,
            textColor
        );
        GlStateManager.popMatrix();
    }

    private void drawBorder(int color) {
        Gui.drawRect(this.xPosition, this.yPosition, this.xPosition + this.width, this.yPosition + 1, color);
        Gui.drawRect(this.xPosition, this.yPosition + this.height - 1, this.xPosition + this.width, this.yPosition + this.height, color);
        Gui.drawRect(this.xPosition, this.yPosition, this.xPosition + 1, this.yPosition + this.height, color);
        Gui.drawRect(this.xPosition + this.width - 1, this.yPosition, this.xPosition + this.width, this.yPosition + this.height, color);
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
