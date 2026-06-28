package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.gui.theme.UiTheme;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiScreen;
import com.paraguacraft.pvp.core.ModLang;
import org.lwjgl.input.Keyboard;

import java.io.IOException;

public class GuiMusicHudOptions extends GuiScreen {

    private static final int ROWS = 2;

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        drawRect(0, 0, width, height, 0x99000000);
        int px = width / 2 - 160;
        int py = height / 2 - 70;
        Gui.drawRect(px, py, px + 320, py + 140, 0xCC0A0C14);
        FontRenderer fr = fontRendererObj;
        fr.drawStringWithShadow(ModLang.format("paraguacraft.music_hud.title"), px + 16, py + 12, UiTheme.ACCENT);
        for (int i = 0; i < ROWS; i++) {
            int rowY = py + 44 + i * 28;
            Gui.drawRect(px + 12, rowY, px + 308, rowY + 20, 0x44000000);
            fr.drawStringWithShadow(rowLabel(i), px + 20, rowY + 6, UiTheme.TEXT);
            fr.drawStringWithShadow(rowValue(i), px + 280 - fr.getStringWidth(rowValue(i)), rowY + 6, rowColor(i));
        }
        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    @Override
    protected void mouseClicked(int mouseX, int mouseY, int mouseButton) throws IOException {
        if (mouseButton != 0) {
            return;
        }
        int px = width / 2 - 160;
        int py = height / 2 - 70;
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
            case 0: return ModLang.format("paraguacraft.music_hud.album_art");
            case 1: return ModLang.format("paraguacraft.music_hud.alpha");
            default: return "";
        }
    }

    private static String rowValue(int i) {
        switch (i) {
            case 0:
                return ModLang.format(ModConfig.showMusicAlbumArt ? "paraguacraft.menu.on" : "paraguacraft.menu.off");
            case 1:
                return ModConfig.musicHudAlphaLabel();
            default:
                return "";
        }
    }

    private static int rowColor(int i) {
        if (i == 0) {
            return ModConfig.showMusicAlbumArt ? 0xFF22CC66 : 0xFFCC4444;
        }
        return UiTheme.ACCENT;
    }

    private static void toggleRow(int i) {
        switch (i) {
            case 0:
                ModConfig.showMusicAlbumArt = !ModConfig.showMusicAlbumArt;
                break;
            case 1:
                ModConfig.cycleMusicHudAlpha();
                break;
            default:
                break;
        }
    }

    @Override
    public boolean doesGuiPauseGame() {
        return false;
    }
}
