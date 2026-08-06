package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.LauncherIpc;
import com.paraguacraft.pvp.modern.core.ServerContext;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import com.paraguacraft.pvp.modern.hud.HudModuleScale;
import com.paraguacraft.pvp.modern.hud.HudRenderer;
import net.minecraft.client.gui.Click;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;
import org.joml.Matrix3x2fStack;

/**
 * Editar HUD: mover módulos y escalar proporcionalmente (solo manijas de esquina).
 */
public class GuiEditHudScreen extends Screen {

    private static final int HANDLE = 5;
    private static final int HIT = 8;

    private final Screen parent;
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

    public GuiEditHudScreen(Screen parent) {
        super(Text.literal("Editar HUD"));
        this.parent = parent;
    }

    @Override
    protected void init() {
        addDrawableChild(ButtonWidget.builder(Text.literal("Guardar y volver"), b -> {
            ModernConfig.save();
            client.setScreen(parent);
        }).dimensions(width / 2 - 80, height - 28, 160, 20).build());
    }

    @Override
    public void renderBackground(DrawContext context, int mouseX, int mouseY, float delta) {
        /* Mundo visible detras del editor. */
    }

    @Override
    public void render(DrawContext ctx, int mouseX, int mouseY, float delta) {
        HudRenderer.renderEditing(ctx);
        ctx.fill(0, 0, width, height, 0x44000000);
        ctx.drawCenteredTextWithShadow(textRenderer, Text.literal("Modo Edicion Paraguacraft"), width / 2, 12, UiTheme.accent());
        ctx.drawCenteredTextWithShadow(
            textRenderer,
            Text.literal("Arrastra el modulo para mover · esquinas blancas = tamaño (proporcional)"),
            width / 2,
            26,
            UiTheme.textDim()
        );
        if (boxId >= 0 && mode == 2) {
            ctx.drawCenteredTextWithShadow(
                textRenderer,
                Text.literal("Escala: " + HudModuleScale.get(boxId) + "%"),
                width / 2,
                42,
                UiTheme.accent()
            );
        }

        float ui = Math.max(0.5f, ModernConfig.uiScaleFactor());
        Matrix3x2fStack matrices = ctx.getMatrices();
        matrices.pushMatrix();
        if (ui != 1f) {
            matrices.scale(ui, ui);
        }

        if (ModernConfig.showFps) {
            drawBox(ctx, 0, ModernConfig.fpsX, ModernConfig.fpsY, 55, 10);
        }
        if (ModernConfig.showPing) {
            drawBox(ctx, 1, ModernConfig.pingX, ModernConfig.pingY, 70, 10);
        }
        if (ModernConfig.showCps) {
            drawBox(ctx, 2, ModernConfig.cpsX, ModernConfig.cpsY, 50, 10);
        }
        if (ModernConfig.showKeystrokes) {
            drawBox(ctx, 3, ModernConfig.keysX, ModernConfig.keysY, 68, 68);
        }
        if (ModernConfig.showArmor) {
            drawBox(ctx, 4, ModernConfig.armorX, ModernConfig.armorY, 45, HudRenderer.armorPanelHeight(client));
        }
        if (ModernConfig.showHeldItem) {
            drawBox(ctx, 5, ModernConfig.heldX, ModernConfig.heldY, 130, 40);
        }
        if (ModernConfig.showBedwarsResources) {
            drawBox(ctx, 6, ModernConfig.bwResX, ModernConfig.bwResY, 42, 68);
        }
        if (ModernConfig.showHardwareHud) {
            LauncherIpc.Snapshot snap = LauncherIpc.get();
            drawBox(ctx, 7, ModernConfig.hardwareHudX, ModernConfig.hardwareHudY,
                HudRenderer.hardwarePanelWidth(snap), HudRenderer.hardwarePanelHeight(snap));
        }
        if (ModernConfig.showBlockCount) {
            drawBox(ctx, 8, ModernConfig.blocksX, ModernConfig.blocksY, 36, 16);
        }
        if (ModernConfig.showPotions) {
            drawBox(ctx, 9, ModernConfig.potionX, ModernConfig.potionY, 120, HudRenderer.potionPanelHeight(client));
        }
        if (ModernConfig.showCoords) {
            drawBox(ctx, 10, ModernConfig.coordsX, ModernConfig.coordsY, 130, 10);
        }
        if (ModernConfig.showCompass) {
            int screenW = (int) (width / ui);
            drawBox(ctx, 11, screenW / 2 - 110, ModernConfig.compassY, 220, 16);
        }
        if (ModernConfig.comboCounter) {
            drawBox(ctx, 12, ModernConfig.comboX, ModernConfig.comboY, 70, 10);
        }
        if (ModernConfig.showMusicHud) {
            LauncherIpc.Snapshot snap = LauncherIpc.get();
            drawBox(ctx, 13, ModernConfig.musicHudX, ModernConfig.musicHudY,
                HudRenderer.musicPanelWidth(snap, true), HudRenderer.musicPanelHeight(snap, true));
        }
        if (ModernConfig.showCombatStatsHud) {
            drawBox(ctx, 14, ModernConfig.combatStatsX, ModernConfig.combatStatsY, 114, 46);
        }
        if (ModernConfig.showGameModeHud) {
            drawBox(ctx, 15, ModernConfig.gameModeHudX, ModernConfig.gameModeHudY, 90, 10);
        }
        if (ModernConfig.showBridgeTimer) {
            drawBox(ctx, 16, ModernConfig.bridgeTimerX, ModernConfig.bridgeTimerY, 80, 10);
        }
        if (ModernConfig.reachDisplay && ServerContext.reachDisplayAllowed(client)) {
            drawBox(ctx, 17, ModernConfig.reachDisplayX, ModernConfig.reachDisplayY, 70, 10);
        }
        if (ModernConfig.showServerHud) {
            drawBox(ctx, 18, ModernConfig.serverHudX, ModernConfig.serverHudY, 120, 10);
        }

        matrices.popMatrix();
        super.render(ctx, mouseX, mouseY, delta);
    }

    private void drawBox(DrawContext ctx, int id, int x, int y, int w, int h) {
        float s = HudModuleScale.factor(HudModuleScale.get(id));
        int sw = Math.max(1, Math.round(w * s));
        int sh = Math.max(1, Math.round(h * s));
        int color = (boxId == id) ? 0xAA00E5FF : 0x8800E5FF;
        ctx.fill(x - 1, y - 1, x + sw + 1, y + sh + 1, color);
        ctx.fill(x, y, x + sw, y + 1, 0xFFFFFFFF);
        ctx.fill(x, y + sh - 1, x + sw, y + sh, 0xFFFFFFFF);
        ctx.fill(x, y, x + 1, y + sh, 0xFFFFFFFF);
        ctx.fill(x + sw - 1, y, x + sw, y + sh, 0xFFFFFFFF);
        // Solo esquinas (no laterales)
        drawHandle(ctx, x, y);
        drawHandle(ctx, x + sw, y);
        drawHandle(ctx, x + sw, y + sh);
        drawHandle(ctx, x, y + sh);
    }

    private void drawHandle(DrawContext ctx, int cx, int cy) {
        int h = HANDLE;
        ctx.fill(cx - h / 2, cy - h / 2, cx + h / 2 + 1, cy + h / 2 + 1, 0xFFFFFFFF);
        ctx.fill(cx - h / 2 + 1, cy - h / 2 + 1, cx + h / 2, cy + h / 2, 0xFF222222);
    }

    @Override
    public boolean shouldPause() {
        return false;
    }

    @Override
    public boolean mouseClicked(Click click, boolean doubled) {
        if (click.buttonInfo().button() != 0) {
            return super.mouseClicked(click, doubled);
        }
        float ui = Math.max(0.5f, ModernConfig.uiScaleFactor());
        int mx = (int) (click.x() / ui);
        int my = (int) (click.y() / ui);

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
            if (corner == 0) {
                anchorX = r[0] + sw;
                anchorY = r[1] + sh;
            } else if (corner == 1) {
                anchorX = r[0];
                anchorY = r[1] + sh;
            } else if (corner == 2) {
                anchorX = r[0];
                anchorY = r[1];
            } else {
                anchorX = r[0] + sw;
                anchorY = r[1];
            }
            startDist = dist(mx, my, anchorX, anchorY);
            if (startDist < 4f) {
                startDist = 4f;
            }
            return true;
        }

        for (int id = 18; id >= 0; id--) {
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
                return true;
            }
        }
        return super.mouseClicked(click, doubled);
    }

    @Override
    public boolean mouseReleased(Click click) {
        if (click.buttonInfo().button() == 0 && mode != 0) {
            ModernConfig.save();
            mode = 0;
            boxId = -1;
            corner = -1;
            return true;
        }
        return super.mouseReleased(click);
    }

    @Override
    public boolean mouseDragged(Click click, double offsetX, double offsetY) {
        if (boxId < 0 || click.buttonInfo().button() != 0) {
            return super.mouseDragged(click, offsetX, offsetY);
        }
        float ui = Math.max(0.5f, ModernConfig.uiScaleFactor());
        int mx = (int) (click.x() / ui);
        int my = (int) (click.y() / ui);

        if (mode == 1) {
            setPos(boxId, mx - dragX, my - dragY);
            return true;
        }
        if (mode == 2) {
            float d = dist(mx, my, anchorX, anchorY);
            float ratio = d / startDist;
            int newScale = HudModuleScale.clamp(Math.round(startScale * ratio));
            HudModuleScale.set(boxId, newScale);
            float s = HudModuleScale.factor(newScale);
            int sw = Math.max(1, Math.round(baseW * s));
            int sh = Math.max(1, Math.round(baseH * s));
            if (corner == 0) {
                setPos(boxId, anchorX - sw, anchorY - sh);
            } else if (corner == 1) {
                setPos(boxId, anchorX, anchorY - sh);
            } else if (corner == 2) {
                setPos(boxId, anchorX, anchorY);
            } else {
                setPos(boxId, anchorX - sw, anchorY);
            }
            return true;
        }
        return super.mouseDragged(click, offsetX, offsetY);
    }

    private int hitHandle(int mx, int my) {
        for (int id = 0; id <= 18; id++) {
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
        return switch (id) {
            case 0 -> ModernConfig.showFps;
            case 1 -> ModernConfig.showPing;
            case 2 -> ModernConfig.showCps;
            case 3 -> ModernConfig.showKeystrokes;
            case 4 -> ModernConfig.showArmor;
            case 5 -> ModernConfig.showHeldItem;
            case 6 -> ModernConfig.showBedwarsResources;
            case 7 -> ModernConfig.showHardwareHud;
            case 8 -> ModernConfig.showBlockCount;
            case 9 -> ModernConfig.showPotions;
            case 10 -> ModernConfig.showCoords;
            case 11 -> ModernConfig.showCompass;
            case 12 -> ModernConfig.comboCounter;
            case 13 -> ModernConfig.showMusicHud;
            case 14 -> ModernConfig.showCombatStatsHud;
            case 15 -> ModernConfig.showGameModeHud;
            case 16 -> ModernConfig.showBridgeTimer;
            case 17 -> ModernConfig.reachDisplay && ServerContext.reachDisplayAllowed(client);
            case 18 -> ModernConfig.showServerHud;
            default -> false;
        };
    }

    /** x, y, baseW, baseH */
    private int[] rectOf(int id) {
        float ui = Math.max(0.5f, ModernConfig.uiScaleFactor());
        int screenW = (int) (width / ui);
        LauncherIpc.Snapshot snap;
        return switch (id) {
            case 0 -> new int[] {ModernConfig.fpsX, ModernConfig.fpsY, 55, 10};
            case 1 -> new int[] {ModernConfig.pingX, ModernConfig.pingY, 70, 10};
            case 2 -> new int[] {ModernConfig.cpsX, ModernConfig.cpsY, 50, 10};
            case 3 -> new int[] {ModernConfig.keysX, ModernConfig.keysY, 68, 68};
            case 4 -> new int[] {ModernConfig.armorX, ModernConfig.armorY, 45, HudRenderer.armorPanelHeight(client)};
            case 5 -> new int[] {ModernConfig.heldX, ModernConfig.heldY, 130, 40};
            case 6 -> new int[] {ModernConfig.bwResX, ModernConfig.bwResY, 42, 68};
            case 7 -> {
                snap = LauncherIpc.get();
                yield new int[] {
                    ModernConfig.hardwareHudX, ModernConfig.hardwareHudY,
                    HudRenderer.hardwarePanelWidth(snap), HudRenderer.hardwarePanelHeight(snap)
                };
            }
            case 8 -> new int[] {ModernConfig.blocksX, ModernConfig.blocksY, 36, 16};
            case 9 -> new int[] {ModernConfig.potionX, ModernConfig.potionY, 120, HudRenderer.potionPanelHeight(client)};
            case 10 -> new int[] {ModernConfig.coordsX, ModernConfig.coordsY, 130, 10};
            case 11 -> new int[] {screenW / 2 - 110, ModernConfig.compassY, 220, 16};
            case 12 -> new int[] {ModernConfig.comboX, ModernConfig.comboY, 70, 10};
            case 13 -> {
                snap = LauncherIpc.get();
                yield new int[] {
                    ModernConfig.musicHudX, ModernConfig.musicHudY,
                    HudRenderer.musicPanelWidth(snap, true), HudRenderer.musicPanelHeight(snap, true)
                };
            }
            case 14 -> new int[] {ModernConfig.combatStatsX, ModernConfig.combatStatsY, 114, 46};
            case 15 -> new int[] {ModernConfig.gameModeHudX, ModernConfig.gameModeHudY, 90, 10};
            case 16 -> new int[] {ModernConfig.bridgeTimerX, ModernConfig.bridgeTimerY, 80, 10};
            case 17 -> new int[] {ModernConfig.reachDisplayX, ModernConfig.reachDisplayY, 70, 10};
            case 18 -> new int[] {ModernConfig.serverHudX, ModernConfig.serverHudY, 120, 10};
            default -> new int[] {0, 0, 10, 10};
        };
    }

    private void setPos(int id, int x, int y) {
        switch (id) {
            case 0 -> { ModernConfig.fpsX = x; ModernConfig.fpsY = y; }
            case 1 -> { ModernConfig.pingX = x; ModernConfig.pingY = y; }
            case 2 -> { ModernConfig.cpsX = x; ModernConfig.cpsY = y; }
            case 3 -> { ModernConfig.keysX = x; ModernConfig.keysY = y; }
            case 4 -> { ModernConfig.armorX = x; ModernConfig.armorY = y; }
            case 5 -> { ModernConfig.heldX = x; ModernConfig.heldY = y; }
            case 6 -> { ModernConfig.bwResX = x; ModernConfig.bwResY = y; }
            case 7 -> { ModernConfig.hardwareHudX = x; ModernConfig.hardwareHudY = y; }
            case 8 -> { ModernConfig.blocksX = x; ModernConfig.blocksY = y; }
            case 9 -> { ModernConfig.potionX = x; ModernConfig.potionY = y; }
            case 10 -> { ModernConfig.coordsX = x; ModernConfig.coordsY = y; }
            case 11 -> { ModernConfig.compassY = y; }
            case 12 -> { ModernConfig.comboX = x; ModernConfig.comboY = y; }
            case 13 -> { ModernConfig.musicHudX = x; ModernConfig.musicHudY = y; }
            case 14 -> { ModernConfig.combatStatsX = x; ModernConfig.combatStatsY = y; }
            case 15 -> { ModernConfig.gameModeHudX = x; ModernConfig.gameModeHudY = y; }
            case 16 -> { ModernConfig.bridgeTimerX = x; ModernConfig.bridgeTimerY = y; }
            case 17 -> { ModernConfig.reachDisplayX = x; ModernConfig.reachDisplayY = y; }
            case 18 -> { ModernConfig.serverHudX = x; ModernConfig.serverHudY = y; }
            default -> {}
        }
    }

    private static float dist(int x1, int y1, int x2, int y2) {
        float dx = x1 - x2;
        float dy = y1 - y2;
        return (float) Math.sqrt(dx * dx + dy * dy);
    }
}
