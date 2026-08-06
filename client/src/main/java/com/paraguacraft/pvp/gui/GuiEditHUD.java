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
        String hint = "Arrastra las cajas · click derecho: escala UI (" + ModConfig.uiScaleLabel() + ")";
        fr.drawStringWithShadow(hint, this.width / 2 - fr.getStringWidth(hint) / 2, 40, UiTheme.TEXT_DIM);

        float s = ModConfig.uiScaleFactor();
        int sx = (int) (mouseX / s);
        int sy = (int) (mouseY / s);

        ScaledResolution sr = new ScaledResolution(mc);
        int box = UiTheme.ACCENT_DIM;

        net.minecraft.client.renderer.GlStateManager.pushMatrix();
        if (s != 1.0f) {
            net.minecraft.client.renderer.GlStateManager.scale(s, s, 1.0f);
        }

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
            int compassX = (int) ((sr.getScaledWidth() / s) / 2) - 100;
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
        if (ModConfig.showBlockCount) {
            drawRect(ModConfig.blocksX - 2, ModConfig.blocksY - 2, ModConfig.blocksX + 40, ModConfig.blocksY + 18, box);
        }
        if (ModConfig.reachDisplay) drawRect(ModConfig.reachDisplayX - 2, ModConfig.reachDisplayY - 2, ModConfig.reachDisplayX + 70, ModConfig.reachDisplayY + 10, box);
        if (ModConfig.comboCounter) drawRect(ModConfig.comboDisplayX - 2, ModConfig.comboDisplayY - 2, ModConfig.comboDisplayX + 70, ModConfig.comboDisplayY + 10, box);

        net.minecraft.client.renderer.GlStateManager.popMatrix();

        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    @Override
    protected void mouseClicked(int mouseX, int mouseY, int mouseButton) {
        if (mouseButton == 1) {
            ModConfig.cycleUiScale();
            ModConfig.save();
            return;
        }
        if (mouseButton == 0) {
            float s = ModConfig.uiScaleFactor();
            int mx = (int) (mouseX / s);
            int my = (int) (mouseY / s);
            ScaledResolution sr = new ScaledResolution(mc);
            int compassCenter = (int) ((sr.getScaledWidth() / s) / 2) - 100;

            if (ModConfig.showFPS && isHover(mx, my, ModConfig.fpsX, ModConfig.fpsY, 55, 10)) {
                dragging = 0; dragX = mx - ModConfig.fpsX; dragY = my - ModConfig.fpsY;
            } else if (ModConfig.showPing && isHover(mx, my, ModConfig.pingX, ModConfig.pingY, 60, 10)) {
                dragging = 1; dragX = mx - ModConfig.pingX; dragY = my - ModConfig.pingY;
            } else if (ModConfig.showCPS && isHover(mx, my, ModConfig.cpsX, ModConfig.cpsY, 50, 10)) {
                dragging = 2; dragX = mx - ModConfig.cpsX; dragY = my - ModConfig.cpsY;
            } else if (ModConfig.showKeystrokes && isHover(mx, my, ModConfig.keysX, ModConfig.keysY, 68, 68)) {
                dragging = 3; dragX = mx - ModConfig.keysX; dragY = my - ModConfig.keysY;
            } else if (ModConfig.showArmor && isHover(mx, my, ModConfig.armorX, ModConfig.armorY, 45, 65)) {
                dragging = 4; dragX = mx - ModConfig.armorX; dragY = my - ModConfig.armorY;
            } else if (ModConfig.showPotions && isHover(mx, my, ModConfig.potionX, ModConfig.potionY, 120, 40)) {
                dragging = 5; dragX = mx - ModConfig.potionX; dragY = my - ModConfig.potionY;
            } else if (ModConfig.showCoords && isHover(mx, my, ModConfig.coordsX, ModConfig.coordsY, 100, 10)) {
                dragging = 6; dragX = mx - ModConfig.coordsX; dragY = my - ModConfig.coordsY;
            } else if (ModConfig.showHeldItem && isHover(mx, my, ModConfig.heldX, ModConfig.heldY, 130, 40)) {
                dragging = 7; dragX = mx - ModConfig.heldX; dragY = my - ModConfig.heldY;
            } else if (ModConfig.showServerHUD && !mc.isIntegratedServerRunning() && isHover(mx, my, ModConfig.serverX, ModConfig.serverY, 150, 24)) {
                dragging = 8; dragX = mx - ModConfig.serverX; dragY = my - ModConfig.serverY;
            } else if (ModConfig.showCompass && isHover(mx, my, compassCenter, ModConfig.compassY, 200, 20)) {
                dragging = 9; dragX = 0; dragY = my - ModConfig.compassY;
            } else if ((ModConfig.showHardwareHud || ModConfig.showMusicHud)
                && isHover(mx, my, ModConfig.overlayHudX, ModConfig.overlayHudY,
                Math.max(ModConfig.overlayHudW, AdvancedHud.overlayPanelWidth(LauncherIpc.get())),
                Math.max(58, AdvancedHud.overlayPanelHeight(LauncherIpc.get())))) {
                dragging = 10; dragX = mx - ModConfig.overlayHudX; dragY = my - ModConfig.overlayHudY;
            } else if (ModConfig.showBedwarsResources
                && isHover(mx, my, ModConfig.bwResX, ModConfig.bwResY, AdvancedHud.bwPanelW(), AdvancedHud.bwPanelH())) {
                dragging = 11; dragX = mx - ModConfig.bwResX; dragY = my - ModConfig.bwResY;
            } else if (ModConfig.showBlockCount && isHover(mx, my, ModConfig.blocksX, ModConfig.blocksY, 40, 18)) {
                dragging = 14; dragX = mx - ModConfig.blocksX; dragY = my - ModConfig.blocksY;
            } else if (ModConfig.reachDisplay && isHover(mx, my, ModConfig.reachDisplayX, ModConfig.reachDisplayY, 70, 10)) {
                dragging = 12; dragX = mx - ModConfig.reachDisplayX; dragY = my - ModConfig.reachDisplayY;
            } else if (ModConfig.comboCounter && isHover(mx, my, ModConfig.comboDisplayX, ModConfig.comboDisplayY, 70, 10)) {
                dragging = 13; dragX = mx - ModConfig.comboDisplayX; dragY = my - ModConfig.comboDisplayY;
            }
        }
    }

    @Override
    protected void mouseClickMove(int mouseX, int mouseY, int clickedMouseButton, long timeSinceLastClick) {
        float s = ModConfig.uiScaleFactor();
        int mx = (int) (mouseX / s);
        int my = (int) (mouseY / s);
        if (dragging == 0) { ModConfig.fpsX = mx - dragX; ModConfig.fpsY = my - dragY; }
        else if (dragging == 1) { ModConfig.pingX = mx - dragX; ModConfig.pingY = my - dragY; }
        else if (dragging == 2) { ModConfig.cpsX = mx - dragX; ModConfig.cpsY = my - dragY; }
        else if (dragging == 3) { ModConfig.keysX = mx - dragX; ModConfig.keysY = my - dragY; }
        else if (dragging == 4) { ModConfig.armorX = mx - dragX; ModConfig.armorY = my - dragY; }
        else if (dragging == 5) { ModConfig.potionX = mx - dragX; ModConfig.potionY = my - dragY; }
        else if (dragging == 6) { ModConfig.coordsX = mx - dragX; ModConfig.coordsY = my - dragY; }
        else if (dragging == 7) { ModConfig.heldX = mx - dragX; ModConfig.heldY = my - dragY; }
        else if (dragging == 8) { ModConfig.serverX = mx - dragX; ModConfig.serverY = my - dragY; }
        else if (dragging == 9) { ModConfig.compassY = my - dragY; }
        else if (dragging == 10) { ModConfig.overlayHudX = mx - dragX; ModConfig.overlayHudY = my - dragY; }
        else if (dragging == 11) { ModConfig.bwResX = mx - dragX; ModConfig.bwResY = my - dragY; }
        else if (dragging == 12) { ModConfig.reachDisplayX = mx - dragX; ModConfig.reachDisplayY = my - dragY; }
        else if (dragging == 13) { ModConfig.comboDisplayX = mx - dragX; ModConfig.comboDisplayY = my - dragY; }
        else if (dragging == 14) { ModConfig.blocksX = mx - dragX; ModConfig.blocksY = my - dragY; }
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
