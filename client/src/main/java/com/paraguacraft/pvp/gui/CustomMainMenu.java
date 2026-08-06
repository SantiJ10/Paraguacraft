package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.core.TrainingWorldHelper;
import com.paraguacraft.pvp.gui.theme.UiTheme;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.gui.GuiButton;
import net.minecraft.client.gui.GuiMultiplayer;
import net.minecraft.client.gui.GuiOptions;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.gui.GuiSelectWorld;

/**
 * Main Menu estilo Lunar / Badlion — fondo limpio, logo, CFont, botones con easing.
 */
public class CustomMainMenu extends GuiScreen {

    @Override
    public void initGui() {
        int btnW = 220;
        int btnH = 26;
        int startY = this.height / 2 + 10;
        int gap = 26;

        this.buttonList.clear();
        this.buttonList.add(new EasingButton(1, this.width / 2 - btnW / 2, startY, btnW, btnH, "Un jugador"));
        this.buttonList.add(new EasingButton(2, this.width / 2 - btnW / 2, startY + gap, btnW, btnH, "Multijugador"));
        if (ModConfig.pvpTrainingMode || ModConfig.pvpTrainingAutoWorld) {
            this.buttonList.add(new EasingButton(3, this.width / 2 - btnW / 2, startY + gap * 2, btnW, btnH, "Practica PvP (flat)"));
            this.buttonList.add(new EasingButton(0, this.width / 2 - btnW / 2, startY + gap * 3, btnW, btnH, "Opciones"));
            this.buttonList.add(new EasingButton(4, this.width / 2 - btnW / 2, startY + gap * 4, btnW, btnH, "Salir"));
        } else {
            this.buttonList.add(new EasingButton(0, this.width / 2 - btnW / 2, startY + gap * 2, btnW, btnH, "Opciones"));
            this.buttonList.add(new EasingButton(4, this.width / 2 - btnW / 2, startY + gap * 3, btnW, btnH, "Salir"));
        }
    }

    @Override
    protected void actionPerformed(GuiButton button) {
        if (button.id == 1) {
            this.mc.displayGuiScreen(new GuiSelectWorld(this));
        } else if (button.id == 2) {
            this.mc.displayGuiScreen(new GuiMultiplayer(this));
        } else if (button.id == 3) {
            TrainingWorldHelper.openTrainingWorld();
        } else if (button.id == 0) {
            this.mc.displayGuiScreen(new GuiOptions(this, this.mc.gameSettings));
        } else if (button.id == 4) {
            this.mc.shutdown();
        }
    }

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        PanoramaBackground.draw(this, partialTicks);
        this.fontRendererObj.drawStringWithShadow(
            "Paraguacraft Client V2 - 1.8.9",
            8,
            this.height - 12,
            UiTheme.TEXT_DIM
        );
        super.drawScreen(mouseX, mouseY, partialTicks);
    }
}
