package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.core.LauncherIpc;
import com.paraguacraft.pvp.gui.theme.UiTheme;
import com.paraguacraft.pvp.hud.AdvancedHud;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.gui.ScaledResolution;

public class GuiEditHUD extends GuiScreen {
    private int dragging = -1;
    private int dragX = 0, dragY = 0;

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        drawRect(0, 0, this.width, this.height, UiTheme.OVERLAY);

        FontRenderer fr = this.fontRendererObj;
        String title = "Modo Edicion Paraguacraft";
        fr.drawStringWithShadow(title, this.width / 2 - fr.getStringWidth(title) / 2, 18, UiTheme.ACCENT);
        String hint = "Arrastra las cajas celestes para reposicionar los HUDs";
        fr.drawStringWithShadow(hint, this.width / 2 - fr.getStringWidth(hint) / 2, 40, UiTheme.TEXT_DIM);

        ScaledResolution sr = new ScaledResolution(mc);
        int box = UiTheme.ACCENT_DIM;

        if (ModConfig.showFPS) drawRect(ModConfig.fpsX - 2, ModConfig.fpsY - 2, ModConfig.fpsX + 55, ModConfig.fpsY + 10, box);
        if (ModConfig.showPing) drawRect(ModConfig.pingX - 2, ModConfig.pingY - 2, ModConfig.pingX + 60, ModConfig.pingY + 10, box);
        if (ModConfig.showCPS) drawRect(ModConfig.cpsX - 2, ModConfig.cpsY - 2, ModConfig.cpsX + 50, ModConfig.cpsY + 10, box);
        if (ModConfig.showKeystrokes) drawRect(ModConfig.keysX - 2, ModConfig.keysY - 2, ModConfig.keysX + 68, ModConfig.keysY + 68, box);
        if (ModConfig.showArmor) drawRect(ModConfig.armorX - 2, ModConfig.armorY - 2, ModConfig.armorX + 45, ModConfig.armorY + 65, box);
        if (ModConfig.showPotions) drawRect(ModConfig.potionX - 2, ModConfig.potionY - 2, ModConfig.potionX + 120, ModConfig.potionY + 40, box);
        if (ModConfig.showCoords) drawRect(ModConfig.coordsX - 2, ModConfig.coordsY - 2, ModConfig.coordsX + 100, ModConfig.coordsY + 10, box);
        if (ModConfig.showHeldItem) drawRect(ModConfig.heldX - 2, ModConfig.heldY - 2, ModConfig.heldX + 130, ModConfig.heldY + 40, box);

        if (ModConfig.showServerHUD && !mc.isIntegratedServerRunning()) {
            drawRect(ModConfig.serverX - 2, ModConfig.serverY - 2, ModConfig.serverX + 150, ModConfig.serverY + 24, box);
        }

        if (ModConfig.showCompass) {
            int compassX = (sr.getScaledWidth() / 2) - 100;
            drawRect(compassX - 2, ModConfig.compassY - 2, compassX + 202, ModConfig.compassY + 22, box);
        }
        if (ModConfig.showHardwareHud || ModConfig.showMusicHud) {
            LauncherIpc.Snapshot snap = LauncherIpc.get();
            int oh = Math.max(58, AdvancedHud.overlayPanelHeight(snap));
            int ow = Math.max(ModConfig.overlayHudW, AdvancedHud.overlayPanelWidth(snap));
            drawRect(ModConfig.overlayHudX - 2, ModConfig.overlayHudY - 2,
                ModConfig.overlayHudX + ow, ModConfig.overlayHudY + oh, box);
        }
        if (ModConfig.showBedwarsResources) {
            drawRect(ModConfig.bwResX - 2, ModConfig.bwResY - 2,
                ModConfig.bwResX + AdvancedHud.bwPanelW(), ModConfig.bwResY + AdvancedHud.bwPanelH(), box);
        }

        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    @Override
    protected void mouseClicked(int mouseX, int mouseY, int mouseButton) {
        if (mouseButton == 0) {
            ScaledResolution sr = new ScaledResolution(mc);

            if (ModConfig.showFPS && isHover(mouseX, mouseY, ModConfig.fpsX, ModConfig.fpsY, 55, 10)) {
                dragging = 0; dragX = mouseX - ModConfig.fpsX; dragY = mouseY - ModConfig.fpsY;
            } else if (ModConfig.showPing && isHover(mouseX, mouseY, ModConfig.pingX, ModConfig.pingY, 60, 10)) {
                dragging = 1; dragX = mouseX - ModConfig.pingX; dragY = mouseY - ModConfig.pingY;
            } else if (ModConfig.showCPS && isHover(mouseX, mouseY, ModConfig.cpsX, ModConfig.cpsY, 50, 10)) {
                dragging = 2; dragX = mouseX - ModConfig.cpsX; dragY = mouseY - ModConfig.cpsY;
            } else if (ModConfig.showKeystrokes && isHover(mouseX, mouseY, ModConfig.keysX, ModConfig.keysY, 68, 68)) {
                dragging = 3; dragX = mouseX - ModConfig.keysX; dragY = mouseY - ModConfig.keysY;
            } else if (ModConfig.showArmor && isHover(mouseX, mouseY, ModConfig.armorX, ModConfig.armorY, 45, 65)) {
                dragging = 4; dragX = mouseX - ModConfig.armorX; dragY = mouseY - ModConfig.armorY;
            } else if (ModConfig.showPotions && isHover(mouseX, mouseY, ModConfig.potionX, ModConfig.potionY, 120, 40)) {
                dragging = 5; dragX = mouseX - ModConfig.potionX; dragY = mouseY - ModConfig.potionY;
            } else if (ModConfig.showCoords && isHover(mouseX, mouseY, ModConfig.coordsX, ModConfig.coordsY, 100, 10)) {
                dragging = 6; dragX = mouseX - ModConfig.coordsX; dragY = mouseY - ModConfig.coordsY;
            } else if (ModConfig.showHeldItem && isHover(mouseX, mouseY, ModConfig.heldX, ModConfig.heldY, 130, 40)) {
                dragging = 7; dragX = mouseX - ModConfig.heldX; dragY = mouseY - ModConfig.heldY;
            } else if (ModConfig.showServerHUD && !mc.isIntegratedServerRunning() && isHover(mouseX, mouseY, ModConfig.serverX, ModConfig.serverY, 150, 24)) {
                dragging = 8; dragX = mouseX - ModConfig.serverX; dragY = mouseY - ModConfig.serverY;
            } else if (ModConfig.showCompass) {
                int compassX = (sr.getScaledWidth() / 2) - 100;
                if (isHover(mouseX, mouseY, compassX, ModConfig.compassY, 200, 20)) {
                    dragging = 9; dragX = 0; dragY = mouseY - ModConfig.compassY;
                } else if ((ModConfig.showHardwareHud || ModConfig.showMusicHud)
                    && isHover(mouseX, mouseY, ModConfig.overlayHudX, ModConfig.overlayHudY,
                    Math.max(ModConfig.overlayHudW, AdvancedHud.overlayPanelWidth(LauncherIpc.get())),
                    Math.max(58, AdvancedHud.overlayPanelHeight(LauncherIpc.get())))) {
                    dragging = 10; dragX = mouseX - ModConfig.overlayHudX; dragY = mouseY - ModConfig.overlayHudY;
                } else if (ModConfig.showBedwarsResources
                    && isHover(mouseX, mouseY, ModConfig.bwResX, ModConfig.bwResY, AdvancedHud.bwPanelW(), AdvancedHud.bwPanelH())) {
                    dragging = 11; dragX = mouseX - ModConfig.bwResX; dragY = mouseY - ModConfig.bwResY;
                }
            } else if ((ModConfig.showHardwareHud || ModConfig.showMusicHud)
                && isHover(mouseX, mouseY, ModConfig.overlayHudX, ModConfig.overlayHudY,
                Math.max(ModConfig.overlayHudW, AdvancedHud.overlayPanelWidth(LauncherIpc.get())),
                Math.max(58, AdvancedHud.overlayPanelHeight(LauncherIpc.get())))) {
                dragging = 10; dragX = mouseX - ModConfig.overlayHudX; dragY = mouseY - ModConfig.overlayHudY;
            } else if (ModConfig.showBedwarsResources
                && isHover(mouseX, mouseY, ModConfig.bwResX, ModConfig.bwResY, AdvancedHud.bwPanelW(), AdvancedHud.bwPanelH())) {
                dragging = 11; dragX = mouseX - ModConfig.bwResX; dragY = mouseY - ModConfig.bwResY;
            }
        }
    }

    @Override
    protected void mouseClickMove(int mouseX, int mouseY, int clickedMouseButton, long timeSinceLastClick) {
        if (dragging == 0) { ModConfig.fpsX = mouseX - dragX; ModConfig.fpsY = mouseY - dragY; }
        else if (dragging == 1) { ModConfig.pingX = mouseX - dragX; ModConfig.pingY = mouseY - dragY; }
        else if (dragging == 2) { ModConfig.cpsX = mouseX - dragX; ModConfig.cpsY = mouseY - dragY; }
        else if (dragging == 3) { ModConfig.keysX = mouseX - dragX; ModConfig.keysY = mouseY - dragY; }
        else if (dragging == 4) { ModConfig.armorX = mouseX - dragX; ModConfig.armorY = mouseY - dragY; }
        else if (dragging == 5) { ModConfig.potionX = mouseX - dragX; ModConfig.potionY = mouseY - dragY; }
        else if (dragging == 6) { ModConfig.coordsX = mouseX - dragX; ModConfig.coordsY = mouseY - dragY; }
        else if (dragging == 7) { ModConfig.heldX = mouseX - dragX; ModConfig.heldY = mouseY - dragY; }
        else if (dragging == 8) { ModConfig.serverX = mouseX - dragX; ModConfig.serverY = mouseY - dragY; }
        else if (dragging == 9) { ModConfig.compassY = mouseY - dragY; }
        else if (dragging == 10) { ModConfig.overlayHudX = mouseX - dragX; ModConfig.overlayHudY = mouseY - dragY; }
        else if (dragging == 11) { ModConfig.bwResX = mouseX - dragX; ModConfig.bwResY = mouseY - dragY; }
    }

    @Override
    protected void mouseReleased(int mouseX, int mouseY, int state) { dragging = -1; }

    @Override
    public void onGuiClosed() {
        ModConfig.save();
    }

    private boolean isHover(int mx, int my, int x, int y, int w, int h) {
        return mx >= x && mx <= x + w && my >= y && my <= y + h;
    }

    @Override
    public boolean doesGuiPauseGame() { return false; }
}
