package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.core.QoLKeybinds;
import com.paraguacraft.pvp.gui.theme.UiTheme;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiScreen;
import org.lwjgl.input.Keyboard;

import java.io.IOException;

/** Editor simple de keybinds del cliente. */
public class GuiKeybinds extends GuiScreen {

    private int listeningFor = -1;
    private static final String[] LABELS = {
        "Mod Menu", "Editar HUD", "Toggle Sprint", "Fullbright", "Freelook", "Hypixel Quick Play"
    };

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        drawRect(0, 0, width, height, 0x99000000);
        int px = width / 2 - 150;
        int py = height / 2 - 70;
        Gui.drawRect(px, py, px + 300, py + 188, 0xCC0A0C14);
        FontRenderer fr = fontRendererObj;
        fr.drawStringWithShadow("KEYBINDS", px + 16, py + 12, UiTheme.ACCENT);
        fr.drawStringWithShadow("Click en una fila y presiona tecla — ESC volver", px + 16, py + 28, UiTheme.TEXT_DIM);
        for (int i = 0; i < LABELS.length; i++) {
            int rowY = py + 52 + i * 22;
            String keyName = listeningFor == i
                ? "..."
                : Keyboard.getKeyName(getKey(i));
            fr.drawStringWithShadow(LABELS[i], px + 16, rowY + 4, UiTheme.TEXT);
            fr.drawStringWithShadow(keyName, px + 220, rowY + 4, UiTheme.ACCENT);
            Gui.drawRect(px + 12, rowY, px + 288, rowY + 18, 0x44000000);
        }
        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    @Override
    protected void mouseClicked(int mouseX, int mouseY, int mouseButton) throws IOException {
        if (mouseButton != 0) {
            return;
        }
        int px = width / 2 - 150;
        int py = height / 2 - 70;
        for (int i = 0; i < LABELS.length; i++) {
            int rowY = py + 52 + i * 22;
            if (mouseX >= px + 12 && mouseX <= px + 288 && mouseY >= rowY && mouseY <= rowY + 18) {
                listeningFor = i;
                return;
            }
        }
    }

    @Override
    protected void keyTyped(char typedChar, int keyCode) throws IOException {
        if (keyCode == Keyboard.KEY_ESCAPE) {
            if (listeningFor >= 0) {
                listeningFor = -1;
                return;
            }
            mc.displayGuiScreen(new GuiParaguaMenu());
            return;
        }
        if (listeningFor >= 0 && keyCode > 0) {
            setKey(listeningFor, keyCode);
            ModConfig.save();
            QoLKeybinds.applyFromConfig();
            listeningFor = -1;
            return;
        }
        super.keyTyped(typedChar, keyCode);
    }

    private static int getKey(int index) {
        switch (index) {
            case 0: return ModConfig.keyMenu;
            case 1: return ModConfig.keyEditHud;
            case 2: return ModConfig.keyToggleSprint;
            case 3: return ModConfig.keyFullbright;
            case 4: return ModConfig.keyFreelook;
            case 5: return ModConfig.keyQuickPlay;
            default: return 0;
        }
    }

    private static void setKey(int index, int code) {
        switch (index) {
            case 0: ModConfig.keyMenu = code; break;
            case 1: ModConfig.keyEditHud = code; break;
            case 2: ModConfig.keyToggleSprint = code; break;
            case 3: ModConfig.keyFullbright = code; break;
            case 4: ModConfig.keyFreelook = code; break;
            case 5: ModConfig.keyQuickPlay = code; break;
            default: break;
        }
    }

    @Override
    public boolean doesGuiPauseGame() {
        return false;
    }
}
