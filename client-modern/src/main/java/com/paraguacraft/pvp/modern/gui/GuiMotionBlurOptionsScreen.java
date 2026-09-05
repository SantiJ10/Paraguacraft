package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.gui.Click;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;

/** Opciones Motion Blur: panel oscuro translúcido estilo Lunar (Type + slider Value). */
public class GuiMotionBlurOptionsScreen extends ParaguacraftScreen {

    private boolean dragging;

    public GuiMotionBlurOptionsScreen(Screen parent) {
        super(Text.literal("MOTION BLUR"), parent);
    }

    @Override
    public void render(DrawContext ctx, int mouseX, int mouseY, float delta) {
        renderBackground(ctx, mouseX, mouseY, delta);
        int px = width / 2 - 210;
        int py = height / 2 - 110;
        int pw = 420;
        int ph = 210;
        ctx.fill(px, py, px + pw, py + ph, 0xE0101218);

        ctx.drawText(textRenderer, Text.literal("‹  MOTION BLUR"), px + 16, py + 14, 0xFFFFFFFF, true);
        ctx.drawText(textRenderer, Text.literal("Blur your surroundings when moving."), px + 18, py + 36, 0xFFAAAAAA, false);

        int typeY = py + 70;
        ctx.fill(px + 16, typeY, px + pw - 16, typeY + 28, 0x44000000);
        ctx.drawText(textRenderer, Text.literal("Type"), px + 24, typeY + 9, 0xFFFFFFFF, true);
        String typeLabel = ModernConfig.motionBlurType == 1 ? "V2" : "V1";
        String arrows = "<  " + typeLabel + "  >";
        ctx.drawText(textRenderer, Text.literal(arrows), px + pw - 24 - textRenderer.getWidth(arrows), typeY + 9, 0xFF5B9DFF, true);

        int valY = py + 112;
        ctx.fill(px + 16, valY, px + pw - 16, valY + 40, 0x44000000);
        ctx.drawText(textRenderer, Text.literal("Value"), px + 24, valY + 8, 0xFFFFFFFF, true);
        int trackX = px + 24;
        int trackY = valY + 26;
        int trackW = pw - 48;
        ctx.fill(trackX, trackY, trackX + trackW, trackY + 4, 0xFF2A2E38);
        int knobX = trackX + Math.round(ModernConfig.motionBlurAmount * trackW);
        ctx.fill(trackX, trackY, knobX, trackY + 4, 0xFF5B9DFF);
        ctx.fill(knobX - 5, trackY - 4, knobX + 5, trackY + 8, 0xFF5B9DFF);

        super.render(ctx, mouseX, mouseY, delta);
    }

    @Override
    public boolean mouseClicked(Click click, boolean doubled) {
        if (click.buttonInfo().button() != 0) {
            return super.mouseClicked(click, doubled);
        }
        int px = width / 2 - 210;
        int py = height / 2 - 110;
        int pw = 420;
        double mx = click.x();
        double my = click.y();
        if (mx >= px + 16 && mx <= px + 80 && my >= py + 10 && my <= py + 32) {
            close();
            return true;
        }
        int typeY = py + 70;
        if (mx >= px + 16 && mx <= px + pw - 16 && my >= typeY && my <= typeY + 28) {
            ModernConfig.motionBlurType = ModernConfig.motionBlurType == 0 ? 1 : 0;
            ModernConfig.save();
            return true;
        }
        int valY = py + 112;
        if (mx >= px + 16 && mx <= px + pw - 16 && my >= valY && my <= valY + 40) {
            dragging = true;
            applySlider(mx);
            return true;
        }
        return super.mouseClicked(click, doubled);
    }

    @Override
    public boolean mouseDragged(Click click, double offsetX, double offsetY) {
        if (dragging && click.buttonInfo().button() == 0) {
            applySlider(click.x());
            return true;
        }
        return super.mouseDragged(click, offsetX, offsetY);
    }

    @Override
    public boolean mouseReleased(Click click) {
        if (dragging) {
            dragging = false;
            ModernConfig.save();
            return true;
        }
        return super.mouseReleased(click);
    }

    private void applySlider(double mouseX) {
        int px = width / 2 - 210;
        int trackX = px + 24;
        int trackW = 420 - 48;
        float t = (float) ((mouseX - trackX) / trackW);
        if (t < 0f) t = 0f;
        if (t > 1f) t = 1f;
        ModernConfig.motionBlurAmount = t;
        if (t > 0.001f) {
            ModernConfig.motionBlurEnabled = true;
        }
    }
}
