package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.core.ModLang;
import com.paraguacraft.pvp.gui.theme.UiTheme;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiScreen;
import org.lwjgl.input.Keyboard;

import java.io.IOException;

/** Opciones Motion Blur: panel oscuro translúcido estilo Lunar (Type + slider Value). */
public class GuiMotionBlurOptions extends GuiScreen {

    private boolean dragging;

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        drawRect(0, 0, width, height, 0x99000000);
        int px = width / 2 - 210;
        int py = height / 2 - 110;
        int pw = 420;
        int ph = 210;
        Gui.drawRect(px, py, px + pw, py + ph, 0xE0101218);

        FontRenderer fr = fontRendererObj;
        fr.drawStringWithShadow("‹  " + ModLang.format("paraguacraft.motion_blur.title"), px + 16, py + 14, 0xFFFFFFFF);
        fr.drawString(ModLang.format("paraguacraft.motion_blur.desc"), px + 18, py + 36, 0xFFAAAAAA);

        int typeY = py + 70;
        Gui.drawRect(px + 16, typeY, px + pw - 16, typeY + 28, 0x44000000);
        fr.drawStringWithShadow(ModLang.format("paraguacraft.motion_blur.type"), px + 24, typeY + 9, 0xFFFFFFFF);
        String typeLabel = ModConfig.motionBlurType == 1 ? "V2" : "V1";
        int typeRight = px + pw - 24;
        fr.drawStringWithShadow("<  " + typeLabel + "  >", typeRight - fr.getStringWidth("<  " + typeLabel + "  >"), typeY + 9, 0xFF5B9DFF);

        int valY = py + 112;
        Gui.drawRect(px + 16, valY, px + pw - 16, valY + 40, 0x44000000);
        fr.drawStringWithShadow(ModLang.format("paraguacraft.motion_blur.value"), px + 24, valY + 8, 0xFFFFFFFF);
        int trackX = px + 24;
        int trackY = valY + 26;
        int trackW = pw - 48;
        Gui.drawRect(trackX, trackY, trackX + trackW, trackY + 4, 0xFF2A2E38);
        int knobX = trackX + Math.round(ModConfig.motionBlurAmount * trackW);
        Gui.drawRect(trackX, trackY, knobX, trackY + 4, 0xFF5B9DFF);
        Gui.drawRect(knobX - 5, trackY - 4, knobX + 5, trackY + 8, 0xFF5B9DFF);

        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    @Override
    protected void mouseClicked(int mouseX, int mouseY, int mouseButton) throws IOException {
        if (mouseButton != 0) {
            return;
        }
        int px = width / 2 - 210;
        int py = height / 2 - 110;
        int pw = 420;
        if (mouseX >= px + 16 && mouseX <= px + 80 && mouseY >= py + 10 && mouseY <= py + 32) {
            mc.displayGuiScreen(new GuiParaguaMenu());
            return;
        }
        int typeY = py + 70;
        if (mouseX >= px + 16 && mouseX <= px + pw - 16 && mouseY >= typeY && mouseY <= typeY + 28) {
            ModConfig.motionBlurType = ModConfig.motionBlurType == 0 ? 1 : 0;
            ModConfig.save();
            return;
        }
        if (hitSlider(mouseX, mouseY)) {
            dragging = true;
            applySlider(mouseX);
        }
    }

    @Override
    protected void mouseClickMove(int mouseX, int mouseY, int clickedMouseButton, long timeSinceLastClick) {
        if (dragging) {
            applySlider(mouseX);
        }
    }

    @Override
    protected void mouseReleased(int mouseX, int mouseY, int state) {
        if (dragging) {
            dragging = false;
            ModConfig.save();
        }
    }

    @Override
    protected void keyTyped(char typedChar, int keyCode) throws IOException {
        if (keyCode == Keyboard.KEY_ESCAPE) {
            mc.displayGuiScreen(new GuiParaguaMenu());
        }
    }

    @Override
    public boolean doesGuiPauseGame() {
        return false;
    }

    private boolean hitSlider(int mx, int my) {
        int px = width / 2 - 210;
        int py = height / 2 - 110;
        int valY = py + 112;
        return mx >= px + 16 && mx <= px + 404 && my >= valY && my <= valY + 40;
    }

    private void applySlider(int mouseX) {
        int px = width / 2 - 210;
        int trackX = px + 24;
        int trackW = 420 - 48;
        float t = (mouseX - trackX) / (float) trackW;
        if (t < 0f) t = 0f;
        if (t > 1f) t = 1f;
        ModConfig.motionBlurAmount = t;
        if (t > 0.001f) {
            ModConfig.motionBlurEnabled = true;
        }
    }
}
