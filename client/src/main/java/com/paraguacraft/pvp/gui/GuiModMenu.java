package com.paraguacraft.pvp.gui;

import net.minecraft.client.gui.GuiButton;
import net.minecraft.client.gui.GuiScreen;
import com.paraguacraft.pvp.modules.ModConfig;

public class GuiModMenu extends GuiScreen {

    @Override
    public void initGui() {
        this.buttonList.clear();
        int startY = this.height / 2 - 85; 
        int btnW = 120;
        int btnH = 20;
        int gap = 24;
        
        int col1 = this.width / 2 - 130;
        int col2 = this.width / 2 + 10;

        this.buttonList.add(new FlatButton(0, col1, startY, btnW, btnH, "FPS: " + (ModConfig.showFPS ? "ON" : "OFF"), false));
        this.buttonList.add(new FlatButton(1, col1, startY + gap, btnW, btnH, "Ping: " + (ModConfig.showPing ? "ON" : "OFF"), false));
        this.buttonList.add(new FlatButton(2, col1, startY + gap * 2, btnW, btnH, "CPS: " + (ModConfig.showCPS ? "ON" : "OFF"), false));
        this.buttonList.add(new FlatButton(3, col1, startY + gap * 3, btnW, btnH, "Keys: " + (ModConfig.showKeystrokes ? "ON" : "OFF"), false));
        this.buttonList.add(new FlatButton(4, col1, startY + gap * 4, btnW, btnH, "NoHurtCam: " + (ModConfig.noHurtCam ? "ON" : "OFF"), false));
        this.buttonList.add(new FlatButton(10, col1, startY + gap * 5, btnW, btnH, "Coords: " + (ModConfig.showCoords ? "ON" : "OFF"), false));
        this.buttonList.add(new FlatButton(12, col1, startY + gap * 6, btnW, btnH, "Armor %: " + (ModConfig.showArmorPercentage ? "ON" : "OFF"), false));

        this.buttonList.add(new FlatButton(5, col2, startY, btnW, btnH, "Armor HUD: " + (ModConfig.showArmor ? "ON" : "OFF"), false));
        this.buttonList.add(new FlatButton(6, col2, startY + gap, btnW, btnH, "Potions HUD: " + (ModConfig.showPotions ? "ON" : "OFF"), false));
        this.buttonList.add(new FlatButton(7, col2, startY + gap * 2, btnW, btnH, "Clear Score: " + (ModConfig.transparentScoreboard ? "ON" : "OFF"), false));
        this.buttonList.add(new FlatButton(8, col2, startY + gap * 3, btnW, btnH, "Mira: " + getCrosshairName(), false));
        this.buttonList.add(new FlatButton(9, col2, startY + gap * 4, btnW, btnH, "Static FOV: " + (ModConfig.dynamicFov ? "ON" : "OFF"), false));
        this.buttonList.add(new FlatButton(11, col2, startY + gap * 5, btnW, btnH, "Toggle Sneak: " + (ModConfig.toggleSneak ? "ON" : "OFF"), false));
    }

    private String getCrosshairName() {
        switch (ModConfig.crosshairMode) {
            case 0: return "Vanilla";
            case 1: return "Cruz Cian";
            case 2: return "Sniper";
            case 3: return "Punto";
            case 4: return "Tightfault";
            default: return "Vanilla";
        }
    }

    @Override
    protected void actionPerformed(GuiButton button) {
        if (button.id == 0) ModConfig.showFPS = !ModConfig.showFPS;
        if (button.id == 1) ModConfig.showPing = !ModConfig.showPing;
        if (button.id == 2) ModConfig.showCPS = !ModConfig.showCPS;
        if (button.id == 3) ModConfig.showKeystrokes = !ModConfig.showKeystrokes;
        if (button.id == 4) ModConfig.noHurtCam = !ModConfig.noHurtCam;
        if (button.id == 5) ModConfig.showArmor = !ModConfig.showArmor;
        if (button.id == 6) ModConfig.showPotions = !ModConfig.showPotions;
        if (button.id == 7) ModConfig.transparentScoreboard = !ModConfig.transparentScoreboard;
        if (button.id == 8) {
            ModConfig.crosshairMode++;
            if (ModConfig.crosshairMode > 4) ModConfig.crosshairMode = 0; 
        }
        if (button.id == 9) ModConfig.dynamicFov = !ModConfig.dynamicFov;
        if (button.id == 10) ModConfig.showCoords = !ModConfig.showCoords;
        if (button.id == 11) {
            ModConfig.toggleSneak = !ModConfig.toggleSneak;
            ModConfig.isSneakingToggled = false; 
        }
        if (button.id == 12) ModConfig.showArmorPercentage = !ModConfig.showArmorPercentage;
        
        // --- LA MAGIA: Guardamos al disco al instante ---
        ModConfig.save();
        
        this.initGui(); 
    }

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        this.drawDefaultBackground();
        this.drawCenteredString(this.fontRendererObj, "PANEL DE CONTROL - PARAGUACRAFT V2.0", this.width / 2, 20, 0x00E5FF);
        super.drawScreen(mouseX, mouseY, partialTicks);
    }
}