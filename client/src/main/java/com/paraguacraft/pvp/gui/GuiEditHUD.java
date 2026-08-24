package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.core.LauncherIpc;
import com.paraguacraft.pvp.gui.theme.UiTheme;
import com.paraguacraft.pvp.hud.AdvancedHud;
import com.paraguacraft.pvp.hud.HUDOverlay;
import com.paraguacraft.pvp.hud.HudModuleScale;
import com.paraguacraft.pvp.modules.ItemTracker;
import com.paraguacraft.pvp.modules.ModConfig;
import com.paraguacraft.pvp.modules.WaypointManager;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.gui.ScaledResolution;
import net.minecraft.client.renderer.GlStateManager;

/**
 * Editar HUD: mover módulos y escalar proporcionalmente (solo manijas de esquina).
 */
public class GuiEditHUD extends GuiScreen {

    private static final int HANDLE = 5;
    private static final int HIT = 8;

    private int mode = 0; // 0 = none, 1 = move, 2 = scale
    private int boxId = -1;
    private int corner = -1; // 0=TL 1=TR 2=BR 3=BL
    private int dragX;
    private int dragY;
    private int startScale;
    private float startDist;
    private int anchorX;
    private int anchorY;
    private int baseW;
    private int baseH;

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        drawRect(0, 0, this.width, this.height, UiTheme.OVERLAY);

        FontRenderer fr = this.fontRendererObj;
        String title = "Modo Edicion Paraguacraft";
        fr.drawStringWithShadow(title, this.width / 2 - fr.getStringWidth(title) / 2, 12, UiTheme.ACCENT);
        String hint = "Arrastra el modulo para mover · esquinas blancas = tamaño (proporcional)";
        fr.drawStringWithShadow(hint, this.width / 2 - fr.getStringWidth(hint) / 2, 28, UiTheme.TEXT_DIM);
        if (boxId >= 0 && mode == 2) {
            String sc = "Escala: " + HudModuleScale.get(boxId) + "%";
            fr.drawStringWithShadow(sc, this.width / 2 - fr.getStringWidth(sc) / 2, 42, UiTheme.ACCENT);
        }

        float ui = Math.max(0.5f, ModConfig.uiScaleFactor());
        int mx = (int) (mouseX / ui);
        int my = (int) (mouseY / ui);

        GlStateManager.pushMatrix();
        if (ui != 1f) {
            GlStateManager.scale(ui, ui, 1f);
        }

        ScaledResolution sr = new ScaledResolution(mc);
        int screenW = (int) (width / ui);

        if (ModConfig.showFPS) drawBox(0, ModConfig.fpsX, ModConfig.fpsY, 55, 10);
        if (ModConfig.showPing) drawBox(1, ModConfig.pingX, ModConfig.pingY, 60, 10);
        if (ModConfig.showCPS) drawBox(2, ModConfig.cpsX, ModConfig.cpsY, 50, 10);
        if (ModConfig.showKeystrokes) drawBox(3, ModConfig.keysX, ModConfig.keysY, HUDOverlay.keystrokesWidth(), HUDOverlay.keystrokesHeight());
        if (ModConfig.showArmor) drawBox(4, ModConfig.armorX, ModConfig.armorY, 45, 65);
        if (ModConfig.showPotions) drawBox(5, ModConfig.potionX, ModConfig.potionY, 120, 48);
        if (ModConfig.showCoords) drawBox(6, ModConfig.coordsX, ModConfig.coordsY, 100, 10);
        if (ModConfig.showHeldItem) drawBox(7, ModConfig.heldX, ModConfig.heldY, 130, 40);
        if (ModConfig.showServerHUD && !mc.isIntegratedServerRunning()) {
            drawBox(8, ModConfig.serverX, ModConfig.serverY, 150, 24);
        }
        if (ModConfig.showCompass) {
            int cx = screenW / 2 - 110;
            drawBox(9, cx, ModConfig.compassY, 220, 16);
        }
        if (ModConfig.showHardwareHud || ModConfig.showMusicHud) {
            LauncherIpc.Snapshot snap = LauncherIpc.get();
            int oh = Math.max(58, AdvancedHud.overlayPanelHeight(snap));
            int ow = Math.max(ModConfig.overlayHudW, AdvancedHud.overlayPanelWidth(snap));
            drawBox(10, ModConfig.overlayHudX, ModConfig.overlayHudY, ow, oh);
        }
        if (ModConfig.showBedwarsResources) {
            drawBox(11, ModConfig.bwResX, ModConfig.bwResY, AdvancedHud.bwPanelW(), AdvancedHud.bwPanelH());
        }
        if (ModConfig.reachDisplay) drawBox(12, ModConfig.reachDisplayX, ModConfig.reachDisplayY, 70, 10);
        if (ModConfig.comboCounter) drawBox(13, ModConfig.comboDisplayX, ModConfig.comboDisplayY, 70, 10);
        if (ModConfig.showBlockCount) drawBox(14, ModConfig.blocksX, ModConfig.blocksY, 40, 18);
        if (ModConfig.itemTracker2d) drawBox(15, ModConfig.itemsX, ModConfig.itemsY, 110, ItemTracker.hudHeight());
        if (ModConfig.showWaypoints) drawBox(16, ModConfig.waypointsX, ModConfig.waypointsY, 140, WaypointManager.hudHeight());

        GlStateManager.popMatrix();
        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    private void drawBox(int id, int x, int y, int w, int h) {
        float s = HudModuleScale.factor(HudModuleScale.get(id));
        int sw = Math.max(1, Math.round(w * s));
        int sh = Math.max(1, Math.round(h * s));
        int color = (boxId == id) ? 0xAA00E5FF : UiTheme.ACCENT_DIM;
        Gui.drawRect(x - 1, y - 1, x + sw + 1, y + sh + 1, color);
        // Borde fino
        Gui.drawRect(x, y, x + sw, y + 1, 0xFFFFFFFF);
        Gui.drawRect(x, y + sh - 1, x + sw, y + sh, 0xFFFFFFFF);
        Gui.drawRect(x, y, x + 1, y + sh, 0xFFFFFFFF);
        Gui.drawRect(x + sw - 1, y, x + sw, y + sh, 0xFFFFFFFF);
        // Solo esquinas (no laterales)
        drawHandle(x, y);
        drawHandle(x + sw, y);
        drawHandle(x + sw, y + sh);
        drawHandle(x, y + sh);
    }

    private void drawHandle(int cx, int cy) {
        int h = HANDLE;
        Gui.drawRect(cx - h / 2, cy - h / 2, cx + h / 2 + 1, cy + h / 2 + 1, 0xFFFFFFFF);
        Gui.drawRect(cx - h / 2 + 1, cy - h / 2 + 1, cx + h / 2, cy + h / 2, 0xFF222222);
    }

    @Override
    protected void mouseClicked(int mouseX, int mouseY, int mouseButton) {
        if (mouseButton != 0) {
            return;
        }
        float ui = Math.max(0.5f, ModConfig.uiScaleFactor());
        int mx = (int) (mouseX / ui);
        int my = (int) (mouseY / ui);

        // Prioridad: manijas de esquina
        int hit = hitHandle(mx, my);
        if (hit >= 0) {
            boxId = hit / 4;
            corner = hit % 4;
            mode = 2;
            int[] r = rectOf(boxId);
            startScale = HudModuleScale.get(boxId);
            baseW = r[2];
            baseH = r[3];
            float s = HudModuleScale.factor(startScale);
            int sw = Math.max(1, Math.round(baseW * s));
            int sh = Math.max(1, Math.round(baseH * s));
            // Ancla = esquina opuesta
            if (corner == 0) { // TL -> BR fijo
                anchorX = r[0] + sw;
                anchorY = r[1] + sh;
            } else if (corner == 1) { // TR -> BL
                anchorX = r[0];
                anchorY = r[1] + sh;
            } else if (corner == 2) { // BR -> TL
                anchorX = r[0];
                anchorY = r[1];
            } else { // BL -> TR
                anchorX = r[0] + sw;
                anchorY = r[1];
            }
            startDist = dist(mx, my, anchorX, anchorY);
            if (startDist < 4f) {
                startDist = 4f;
            }
            return;
        }

        // Cuerpo = mover
        for (int id = 16; id >= 0; id--) {
            if (!isVisible(id)) {
                continue;
            }
            int[] r = rectOf(id);
            float s = HudModuleScale.factor(HudModuleScale.get(id));
            int sw = Math.max(1, Math.round(r[2] * s));
            int sh = Math.max(1, Math.round(r[3] * s));
            if (mx >= r[0] - 2 && mx <= r[0] + sw + 2 && my >= r[1] - 2 && my <= r[1] + sh + 2) {
                mode = 1;
                boxId = id;
                dragX = mx - r[0];
                dragY = my - r[1];
                return;
            }
        }
    }

    @Override
    protected void mouseClickMove(int mouseX, int mouseY, int clickedMouseButton, long timeSinceLastClick) {
        if (boxId < 0) {
            return;
        }
        float ui = Math.max(0.5f, ModConfig.uiScaleFactor());
        int mx = (int) (mouseX / ui);
        int my = (int) (mouseY / ui);

        if (mode == 1) {
            setPos(boxId, mx - dragX, my - dragY);
        } else if (mode == 2) {
            float d = dist(mx, my, anchorX, anchorY);
            float ratio = d / startDist;
            int newScale = HudModuleScale.clamp(Math.round(startScale * ratio));
            HudModuleScale.set(boxId, newScale);
            // Reubicar para mantener esquina opuesta fija
            float s = HudModuleScale.factor(newScale);
            int sw = Math.max(1, Math.round(baseW * s));
            int sh = Math.max(1, Math.round(baseH * s));
            if (corner == 0) { // TL se mueve, BR ancla
                setPos(boxId, anchorX - sw, anchorY - sh);
            } else if (corner == 1) { // TR
                setPos(boxId, anchorX, anchorY - sh);
            } else if (corner == 2) { // BR
                setPos(boxId, anchorX, anchorY);
            } else { // BL
                setPos(boxId, anchorX - sw, anchorY);
            }
        }
    }

    @Override
    protected void mouseReleased(int mouseX, int mouseY, int state) {
        if (mode != 0) {
            ModConfig.save();
        }
        mode = 0;
        boxId = -1;
        corner = -1;
    }

    @Override
    public void onGuiClosed() {
        ModConfig.save();
    }

    private int hitHandle(int mx, int my) {
        for (int id = 0; id <= 16; id++) {
            if (!isVisible(id)) {
                continue;
            }
            int[] r = rectOf(id);
            float s = HudModuleScale.factor(HudModuleScale.get(id));
            int sw = Math.max(1, Math.round(r[2] * s));
            int sh = Math.max(1, Math.round(r[3] * s));
            int[][] corners = {
                {r[0], r[1]},
                {r[0] + sw, r[1]},
                {r[0] + sw, r[1] + sh},
                {r[0], r[1] + sh}
            };
            for (int c = 0; c < 4; c++) {
                if (Math.abs(mx - corners[c][0]) <= HIT && Math.abs(my - corners[c][1]) <= HIT) {
                    return id * 4 + c;
                }
            }
        }
        return -1;
    }

    private boolean isVisible(int id) {
        switch (id) {
            case 0: return ModConfig.showFPS;
            case 1: return ModConfig.showPing;
            case 2: return ModConfig.showCPS;
            case 3: return ModConfig.showKeystrokes;
            case 4: return ModConfig.showArmor;
            case 5: return ModConfig.showPotions;
            case 6: return ModConfig.showCoords;
            case 7: return ModConfig.showHeldItem;
            case 8: return ModConfig.showServerHUD && !mc.isIntegratedServerRunning();
            case 9: return ModConfig.showCompass;
            case 10: return ModConfig.showHardwareHud || ModConfig.showMusicHud;
            case 11: return ModConfig.showBedwarsResources;
            case 12: return ModConfig.reachDisplay;
            case 13: return ModConfig.comboCounter;
            case 14: return ModConfig.showBlockCount;
            case 15: return ModConfig.itemTracker2d;
            case 16: return ModConfig.showWaypoints;
            default: return false;
        }
    }

    /** x,y,baseW,baseH */
    private int[] rectOf(int id) {
        float ui = Math.max(0.5f, ModConfig.uiScaleFactor());
        int screenW = (int) (width / ui);
        switch (id) {
            case 0: return new int[] {ModConfig.fpsX, ModConfig.fpsY, 55, 10};
            case 1: return new int[] {ModConfig.pingX, ModConfig.pingY, 60, 10};
            case 2: return new int[] {ModConfig.cpsX, ModConfig.cpsY, 50, 10};
            case 3: return new int[] {ModConfig.keysX, ModConfig.keysY, HUDOverlay.keystrokesWidth(), HUDOverlay.keystrokesHeight()};
            case 4: return new int[] {ModConfig.armorX, ModConfig.armorY, 45, 65};
            case 5: return new int[] {ModConfig.potionX, ModConfig.potionY, 120, 48};
            case 6: return new int[] {ModConfig.coordsX, ModConfig.coordsY, 100, 10};
            case 7: return new int[] {ModConfig.heldX, ModConfig.heldY, 130, 40};
            case 8: return new int[] {ModConfig.serverX, ModConfig.serverY, 150, 24};
            case 9: return new int[] {screenW / 2 - 110, ModConfig.compassY, 220, 16};
            case 10: {
                LauncherIpc.Snapshot snap = LauncherIpc.get();
                int oh = Math.max(58, AdvancedHud.overlayPanelHeight(snap));
                int ow = Math.max(ModConfig.overlayHudW, AdvancedHud.overlayPanelWidth(snap));
                return new int[] {ModConfig.overlayHudX, ModConfig.overlayHudY, ow, oh};
            }
            case 11: return new int[] {ModConfig.bwResX, ModConfig.bwResY, AdvancedHud.bwPanelW(), AdvancedHud.bwPanelH()};
            case 12: return new int[] {ModConfig.reachDisplayX, ModConfig.reachDisplayY, 70, 10};
            case 13: return new int[] {ModConfig.comboDisplayX, ModConfig.comboDisplayY, 70, 10};
            case 14: return new int[] {ModConfig.blocksX, ModConfig.blocksY, 40, 18};
            case 15: return new int[] {ModConfig.itemsX, ModConfig.itemsY, 140, ItemTracker.hudHeight()};
            case 16: return new int[] {ModConfig.waypointsX, ModConfig.waypointsY, 140, WaypointManager.hudHeight()};
            default: return new int[] {0, 0, 10, 10};
        }
    }

    private void setPos(int id, int x, int y) {
        switch (id) {
            case 0: ModConfig.fpsX = x; ModConfig.fpsY = y; break;
            case 1: ModConfig.pingX = x; ModConfig.pingY = y; break;
            case 2: ModConfig.cpsX = x; ModConfig.cpsY = y; break;
            case 3: ModConfig.keysX = x; ModConfig.keysY = y; break;
            case 4: ModConfig.armorX = x; ModConfig.armorY = y; break;
            case 5: ModConfig.potionX = x; ModConfig.potionY = y; break;
            case 6: ModConfig.coordsX = x; ModConfig.coordsY = y; break;
            case 7: ModConfig.heldX = x; ModConfig.heldY = y; break;
            case 8: ModConfig.serverX = x; ModConfig.serverY = y; break;
            case 9: ModConfig.compassY = y; break;
            case 10: ModConfig.overlayHudX = x; ModConfig.overlayHudY = y; break;
            case 11: ModConfig.bwResX = x; ModConfig.bwResY = y; break;
            case 12: ModConfig.reachDisplayX = x; ModConfig.reachDisplayY = y; break;
            case 13: ModConfig.comboDisplayX = x; ModConfig.comboDisplayY = y; break;
            case 14: ModConfig.blocksX = x; ModConfig.blocksY = y; break;
            case 15: ModConfig.itemsX = x; ModConfig.itemsY = y; break;
            case 16: ModConfig.waypointsX = x; ModConfig.waypointsY = y; break;
            default: break;
        }
    }

    private static float dist(int x1, int y1, int x2, int y2) {
        float dx = x1 - x2;
        float dy = y1 - y2;
        return (float) Math.sqrt(dx * dx + dy * dy);
    }

    @Override
    public boolean doesGuiPauseGame() {
        return false;
    }
}
