package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.gui.theme.UiTheme;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiScreen;
import com.paraguacraft.pvp.core.ModLang;
import org.lwjgl.input.Keyboard;

import java.io.IOException;

public class GuiScoreboardOptions extends GuiScreen {

    private static final int ROWS = 5;

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        drawRect(0, 0, width, height, 0x99000000);
        int px = width / 2 - 160;
        int py = height / 2 - 100;
        Gui.drawRect(px, py, px + 320, py + 200, 0xCC0A0C14);
        FontRenderer fr = fontRendererObj;
        fr.drawStringWithShadow(ModLang.format("paraguacraft.scoreboard.title"), px + 16, py + 12, UiTheme.ACCENT);
        for (int i = 0; i < ROWS; i++) {
            int rowY = py + 44 + i * 28;
            Gui.drawRect(px + 12, rowY, px + 308, rowY + 20, 0x44000000);
            fr.drawStringWithShadow(rowLabel(i), px + 20, rowY + 6, UiTheme.TEXT);
            String state;
            int color;
            if (i == 4) {
                state = ModConfig.scoreboardScale + "%";
                color = UiTheme.ACCENT;
            } else {
                boolean on = rowValue(i);
                state = ModLang.format(on ? "paraguacraft.menu.on" : "paraguacraft.menu.off");
                color = on ? 0xFF22CC66 : 0xFFCC4444;
            }
            fr.drawStringWithShadow(state, px + 280 - fr.getStringWidth(state), rowY + 6, color);
        }
        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    @Override
    protected void mouseClicked(int mouseX, int mouseY, int mouseButton) throws IOException {
        if (mouseButton != 0) {
            return;
        }
        int px = width / 2 - 160;
        int py = height / 2 - 100;
        for (int i = 0; i < ROWS; i++) {
            int rowY = py + 44 + i * 28;
            if (mouseX >= px + 12 && mouseX <= px + 308 && mouseY >= rowY && mouseY <= rowY + 20) {
                toggleRow(i);
                ModConfig.save();
                return;
            }
        }
    }

    @Override
    protected void keyTyped(char typedChar, int keyCode) throws IOException {
        if (keyCode == Keyboard.KEY_ESCAPE) {
            mc.displayGuiScreen(new GuiParaguaMenu());
        }
    }

    private static String rowLabel(int i) {
        switch (i) {
            case 0: return ModLang.format("paraguacraft.scoreboard.transparent");
            case 1: return ModLang.format("paraguacraft.scoreboard.hide_red");
            case 2: return ModLang.format("paraguacraft.scoreboard.hide_stats");
            case 3: return ModLang.format("paraguacraft.scoreboard.shadow");
            case 4: return ModLang.format("paraguacraft.scoreboard.scale");
            default: return "";
        }
    }

    private static boolean rowValue(int i) {
        switch (i) {
            case 0: return ModConfig.scoreboardTransparentBg;
            case 1: return ModConfig.scoreboardHideRedNumbers;
            case 2: return ModConfig.scoreboardHideStats;
            case 3: return ModConfig.scoreboardTextShadow;
            default: return false;
        }
    }

    private static void toggleRow(int i) {
        switch (i) {
            case 0: ModConfig.scoreboardTransparentBg = !ModConfig.scoreboardTransparentBg; break;
            case 1: ModConfig.scoreboardHideRedNumbers = !ModConfig.scoreboardHideRedNumbers; break;
            case 2: ModConfig.scoreboardHideStats = !ModConfig.scoreboardHideStats; break;
            case 3: ModConfig.scoreboardTextShadow = !ModConfig.scoreboardTextShadow; break;
            case 4: ModConfig.cycleScoreboardScale(); break;
            default: break;
        }
    }

    @Override
    public boolean doesGuiPauseGame() {
        return false;
    }
}
