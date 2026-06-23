package com.paraguacraft.pvp.gui;

import net.minecraft.client.gui.GuiButton;
import net.minecraft.client.gui.GuiMultiplayer;
import net.minecraft.client.gui.GuiOptions;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.multiplayer.WorldClient;

public class CustomPauseMenu extends GuiScreen {

    @Override
    public void initGui() {
        this.buttonList.clear();
        int startY = this.height / 2 - 50;
        int btnWidth = 200;
        int btnHeight = 22;
        int spacing = 26;

        this.buttonList.add(new FlatButton(1, this.width / 2 - btnWidth / 2, startY, btnWidth, btnHeight, "Volver al Juego", false));
        // --- NUEVO BOTON DE MODS ACA ---
        this.buttonList.add(new FlatButton(3, this.width / 2 - btnWidth / 2, startY + spacing, btnWidth, btnHeight, "Paraguacraft Mods", false));
        this.buttonList.add(new FlatButton(0, this.width / 2 - btnWidth / 2, startY + spacing * 2, btnWidth, btnHeight, "Opciones", false));
        this.buttonList.add(new FlatButton(2, this.width / 2 - btnWidth / 2, startY + spacing * 3, btnWidth, btnHeight, "Desconectarse", false));
    }

    @Override
    protected void actionPerformed(GuiButton button) {
        if (button.id == 1) {
            this.mc.displayGuiScreen((GuiScreen)null);
            this.mc.setIngameFocus();
        }
        if (button.id == 3) {
            // Aca cambiamos el llamado al nuevo menu
            this.mc.displayGuiScreen(new GuiParaguaMenu()); 
        }
        if (button.id == 0) {
            this.mc.displayGuiScreen(new GuiOptions(this, this.mc.gameSettings));
        }
        if (button.id == 2) {
            boolean isLocal = this.mc.isIntegratedServerRunning();
            button.enabled = false;
            this.mc.theWorld.sendQuittingDisconnectingPacket();
            this.mc.loadWorld((WorldClient)null);
            
            if (isLocal) {
                this.mc.displayGuiScreen(new CustomMainMenu());
            } else {
                this.mc.displayGuiScreen(new GuiMultiplayer(new CustomMainMenu()));
            }
        }
    }

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        this.drawRect(0, 0, this.width, this.height, 0x80000000);
        this.drawCenteredString(this.fontRendererObj, "PAUSA - PARAGUACRAFT V2.0", this.width / 2, this.height / 2 - 80, 0x00E5FF);
        super.drawScreen(mouseX, mouseY, partialTicks);
    }
}