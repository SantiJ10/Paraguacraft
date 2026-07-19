package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.LauncherIpc;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import com.paraguacraft.pvp.modern.hud.HudRenderer;
import net.minecraft.client.gui.Click;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

/** Arrastra elementos del HUD con preview real (solo mods activos). */
public class GuiEditHudScreen extends Screen {

    private static final int ACCENT_BOX = 0x8800E5FF;

    private final Screen parent;
    private int dragging = -1;
    private int dragOffX;
    private int dragOffY;

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
        ctx.drawCenteredTextWithShadow(textRenderer, Text.literal("Arrastra las cajas celestes (solo mods activos)"), width / 2, 26, UiTheme.textDim());

        if (ModernConfig.showFps) {
            drawEditBox(ctx, 0, ModernConfig.fpsX, ModernConfig.fpsY, 55, 10);
        }
        if (ModernConfig.showPing) {
            drawEditBox(ctx, 1, ModernConfig.pingX, ModernConfig.pingY, 70, 10);
        }
        if (ModernConfig.showCps) {
            drawEditBox(ctx, 2, ModernConfig.cpsX, ModernConfig.cpsY, 50, 10);
        }
        if (ModernConfig.showKeystrokes) {
            drawEditBox(ctx, 3, ModernConfig.keysX, ModernConfig.keysY, 68, 68);
        }
        if (ModernConfig.showArmor) {
            drawEditBox(ctx, 4, ModernConfig.armorX, ModernConfig.armorY, 45, HudRenderer.armorPanelHeight(client));
        }
        if (ModernConfig.showPotions) {
            drawEditBox(ctx, 9, ModernConfig.potionX, ModernConfig.potionY, 120, HudRenderer.potionPanelHeight(client));
        }
        if (ModernConfig.showCoords) {
            drawEditBox(ctx, 10, ModernConfig.coordsX, ModernConfig.coordsY, 130, 10);
        }
        if (ModernConfig.showHeldItem) {
            drawEditBox(ctx, 5, ModernConfig.heldX, ModernConfig.heldY, 130, 40);
        }
        if (ModernConfig.showMusicHud) {
            LauncherIpc.Snapshot snap = LauncherIpc.get();
            drawEditBox(ctx, 7, ModernConfig.overlayHudX, ModernConfig.overlayHudY,
                HudRenderer.musicPanelWidth(snap, true), HudRenderer.musicPanelHeight(snap, true));
        }
        if (ModernConfig.showBedwarsResources) {
            drawEditBox(ctx, 6, ModernConfig.bwResX, ModernConfig.bwResY, 42, 68);
        }
        if (ModernConfig.showBlockCount) {
            drawEditBox(ctx, 8, ModernConfig.blocksX, ModernConfig.blocksY, 36, 16);
        }
        if (ModernConfig.showCompass) {
            int cx = width / 2;
            drawEditBox(ctx, 11, cx - 110, ModernConfig.compassY, 220, 16);
        }
        if (ModernConfig.comboCounter) {
            drawEditBox(ctx, 12, ModernConfig.comboX, ModernConfig.comboY, 70, 10);
        }

        super.render(ctx, mouseX, mouseY, delta);
    }

    private void drawEditBox(DrawContext ctx, int id, int x, int y, int w, int h) {
        int color = dragging == id ? 0xAA00E5FF : ACCENT_BOX;
        ctx.fill(x - 2, y - 2, x + w + 2, y + h + 2, color);
    }

    @Override
    public boolean shouldPause() {
        return false;
    }

    @Override
    public boolean mouseClicked(Click click, boolean doubled) {
        if (click.buttonInfo().button() == 0) {
            dragging = hit(click.x(), click.y());
            if (dragging >= 0) {
                int[] pos = posFor(dragging);
                dragOffX = (int) click.x() - pos[0];
                dragOffY = (int) click.y() - pos[1];
                return true;
            }
        }
        return super.mouseClicked(click, doubled);
    }

    @Override
    public boolean mouseReleased(Click click) {
        if (click.buttonInfo().button() == 0 && dragging >= 0) {
            dragging = -1;
            ModernConfig.save();
            return true;
        }
        return super.mouseReleased(click);
    }

    @Override
    public boolean mouseDragged(Click click, double offsetX, double offsetY) {
        if (dragging >= 0 && click.buttonInfo().button() == 0) {
            setPos(dragging, (int) click.x() - dragOffX, (int) click.y() - dragOffY);
            return true;
        }
        return super.mouseDragged(click, offsetX, offsetY);
    }

    private int hit(double mx, double my) {
        if (ModernConfig.showFps && inBox(mx, my, ModernConfig.fpsX, ModernConfig.fpsY, 55, 10)) return 0;
        if (ModernConfig.showPing && inBox(mx, my, ModernConfig.pingX, ModernConfig.pingY, 70, 10)) return 1;
        if (ModernConfig.showCps && inBox(mx, my, ModernConfig.cpsX, ModernConfig.cpsY, 50, 10)) return 2;
        if (ModernConfig.showKeystrokes && inBox(mx, my, ModernConfig.keysX, ModernConfig.keysY, 68, 68)) return 3;
        if (ModernConfig.showArmor && inBox(mx, my, ModernConfig.armorX, ModernConfig.armorY, 45, HudRenderer.armorPanelHeight(client))) return 4;
        if (ModernConfig.showHeldItem && inBox(mx, my, ModernConfig.heldX, ModernConfig.heldY, 130, 40)) return 5;
        if (ModernConfig.showBedwarsResources && inBox(mx, my, ModernConfig.bwResX, ModernConfig.bwResY, 42, 68)) return 6;
        if (ModernConfig.showMusicHud) {
            LauncherIpc.Snapshot snap = LauncherIpc.get();
            if (inBox(mx, my, ModernConfig.overlayHudX, ModernConfig.overlayHudY,
                HudRenderer.musicPanelWidth(snap, true), HudRenderer.musicPanelHeight(snap, true))) {
                return 7;
            }
        }
        if (ModernConfig.showBlockCount && inBox(mx, my, ModernConfig.blocksX, ModernConfig.blocksY, 36, 16)) return 8;
        if (ModernConfig.showPotions && inBox(mx, my, ModernConfig.potionX, ModernConfig.potionY, 120, HudRenderer.potionPanelHeight(client))) return 9;
        if (ModernConfig.showCoords && inBox(mx, my, ModernConfig.coordsX, ModernConfig.coordsY, 130, 10)) return 10;
        if (ModernConfig.showCompass) {
            int cx = width / 2;
            if (inBox(mx, my, cx - 110, ModernConfig.compassY, 220, 16)) return 11;
        }
        if (ModernConfig.comboCounter && inBox(mx, my, ModernConfig.comboX, ModernConfig.comboY, 70, 10)) return 12;
        return -1;
    }

    private static boolean inBox(double mx, double my, int x, int y, int w, int h) {
        return mx >= x - 2 && mx <= x + w + 2 && my >= y - 2 && my <= y + h + 2;
    }

    private int[] posFor(int id) {
        return switch (id) {
            case 0 -> new int[] {ModernConfig.fpsX, ModernConfig.fpsY};
            case 1 -> new int[] {ModernConfig.pingX, ModernConfig.pingY};
            case 2 -> new int[] {ModernConfig.cpsX, ModernConfig.cpsY};
            case 3 -> new int[] {ModernConfig.keysX, ModernConfig.keysY};
            case 4 -> new int[] {ModernConfig.armorX, ModernConfig.armorY};
            case 5 -> new int[] {ModernConfig.heldX, ModernConfig.heldY};
            case 6 -> new int[] {ModernConfig.bwResX, ModernConfig.bwResY};
            case 7 -> new int[] {ModernConfig.overlayHudX, ModernConfig.overlayHudY};
            case 8 -> new int[] {ModernConfig.blocksX, ModernConfig.blocksY};
            case 10 -> new int[] {ModernConfig.coordsX, ModernConfig.coordsY};
            case 11 -> new int[] {width / 2 - 110, ModernConfig.compassY};
            case 12 -> new int[] {ModernConfig.comboX, ModernConfig.comboY};
            default -> new int[] {ModernConfig.potionX, ModernConfig.potionY};
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
            case 7 -> { ModernConfig.overlayHudX = x; ModernConfig.overlayHudY = y; }
            case 8 -> { ModernConfig.blocksX = x; ModernConfig.blocksY = y; }
            case 10 -> { ModernConfig.coordsX = x; ModernConfig.coordsY = y; }
            case 11 -> { ModernConfig.compassY = y; }
            case 12 -> { ModernConfig.comboX = x; ModernConfig.comboY = y; }
            default -> { ModernConfig.potionX = x; ModernConfig.potionY = y; }
        }
    }
}
