package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.Click;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

/** Arrastra elementos del HUD (como 1.8.9). */
public class GuiEditHudScreen extends Screen {

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
    public void render(DrawContext ctx, int mouseX, int mouseY, float delta) {
        ctx.fill(0, 0, width, height, 0x88000000);
        ctx.drawCenteredTextWithShadow(textRenderer, Text.literal("Arrastra los modulos del HUD"), width / 2, 12, UiTheme.accent());
        drawHandle(ctx, 0, ModernConfig.fpsX, ModernConfig.fpsY, "FPS");
        drawHandle(ctx, 1, ModernConfig.pingX, ModernConfig.pingY, "Ping");
        drawHandle(ctx, 2, ModernConfig.cpsX, ModernConfig.cpsY, "CPS");
        drawHandle(ctx, 3, ModernConfig.keysX, ModernConfig.keysY, "Keys");
        drawHandle(ctx, 4, ModernConfig.armorX, ModernConfig.armorY, "Armor");
        drawHandle(ctx, 8, ModernConfig.blocksX, ModernConfig.blocksY, "Bloques");
        drawHandle(ctx, 5, ModernConfig.heldX, ModernConfig.heldY, "Mano");
        drawHandle(ctx, 6, ModernConfig.bwResX, ModernConfig.bwResY, "BW");
        drawHandle(ctx, 7, ModernConfig.overlayHudX, ModernConfig.overlayHudY, "Musica");
        drawHandle(ctx, 9, ModernConfig.potionX, ModernConfig.potionY, "Pociones");
        super.render(ctx, mouseX, mouseY, delta);
    }

    private void drawHandle(DrawContext ctx, int id, int x, int y, String label) {
        int w = Math.max(48, textRenderer.getWidth(label) + 12);
        ctx.fill(x, y, x + w, y + 14, dragging == id ? 0xAA00E5FF : 0x88000000);
        ctx.drawText(textRenderer, Text.literal(label), x + 4, y + 3, 0xFFFFFFFF, true);
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
        for (int i = 0; i < 10; i++) {
            int[] p = posFor(i);
            int w = Math.max(48, textRenderer.getWidth(labelFor(i)) + 12);
            if (mx >= p[0] && mx <= p[0] + w && my >= p[1] && my <= p[1] + 14) {
                return i;
            }
        }
        return -1;
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
            default -> { ModernConfig.potionX = x; ModernConfig.potionY = y; }
        }
    }

    private String labelFor(int id) {
        return switch (id) {
            case 0 -> "FPS";
            case 1 -> "Ping";
            case 2 -> "CPS";
            case 3 -> "Keys";
            case 4 -> "Armor";
            case 5 -> "Mano";
            case 6 -> "BW";
            case 7 -> "Musica";
            case 8 -> "Bloques";
            default -> "Pociones";
        };
    }
}
