package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.gui.theme.UiTheme;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.resources.I18n;
import org.lwjgl.input.Keyboard;

import java.io.IOException;

public class GuiBwResourcesOptions extends GuiScreen {

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        drawRect(0, 0, width, height, 0x99000000);
        int px = width / 2 - 160;
        int py = height / 2 - 50;
        Gui.drawRect(px, py, px + 320, py + 100, 0xCC0A0C14);
        FontRenderer fr = fontRendererObj;
        fr.drawStringWithShadow(I18n.format("paraguacraft.bw_resources.title"), px + 16, py + 12, UiTheme.ACCENT);
        int rowY = py + 44;
        boolean on = ModConfig.bwResTransparentBg;
        Gui.drawRect(px + 12, rowY, px + 308, rowY + 20, 0x44000000);
        fr.drawStringWithShadow(I18n.format("paraguacraft.bw_resources.transparent"), px + 20, rowY + 6, UiTheme.TEXT);
        String state = I18n.format(on ? "paraguacraft.menu.on" : "paraguacraft.menu.off");
        fr.drawStringWithShadow(state, px + 280 - fr.getStringWidth(state), rowY + 6, on ? 0xFF22CC66 : 0xFFCC4444);
        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    @Override
    protected void mouseClicked(int mouseX, int mouseY, int mouseButton) throws IOException {
        if (mouseButton != 0) {
            return;
        }
        int px = width / 2 - 160;
        int py = height / 2 - 50;
        int rowY = py + 44;
        if (mouseX >= px + 12 && mouseX <= px + 308 && mouseY >= rowY && mouseY <= rowY + 20) {
            ModConfig.bwResTransparentBg = !ModConfig.bwResTransparentBg;
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
}
