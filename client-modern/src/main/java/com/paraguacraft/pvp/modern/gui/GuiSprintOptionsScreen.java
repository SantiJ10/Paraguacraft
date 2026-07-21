package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.Click;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;

/** Modos de sprint: Toggle Sprint (estilo Lunar) y Auto Sprint (corre solo). Se pueden combinar. */
public class GuiSprintOptionsScreen extends ParaguacraftScreen {

    private static final int ROWS = 2;
    private static final int ROW_H = 44;

    public GuiSprintOptionsScreen(Screen parent) {
        super(Text.literal("Sprint"), parent);
    }

    @Override
    public void render(DrawContext ctx, int mouseX, int mouseY, float delta) {
        renderBackground(ctx, mouseX, mouseY, delta);
        int px = width / 2 - 160;
        int py = height / 2 - 80;
        ctx.fill(px, py, px + 320, py + 160, 0xCC0A0C14);
        ctx.drawText(textRenderer, Text.literal("Sprint"), px + 16, py + 12, UiTheme.accent(), true);
        for (int i = 0; i < ROWS; i++) {
            int rowY = py + 44 + i * ROW_H;
            ctx.fill(px + 12, rowY, px + 308, rowY + 22, 0x44000000);
            ctx.drawText(textRenderer, Text.literal(rowLabel(i)), px + 20, rowY + 7, UiTheme.TEXT, true);
            String value = rowValue(i);
            ctx.drawText(textRenderer, Text.literal(value), px + 288 - textRenderer.getWidth(value), rowY + 7, rowColor(i), true);
            ctx.drawText(textRenderer, Text.literal(rowDescription(i)), px + 20, rowY + 26, UiTheme.textDim(), false);
        }
        super.render(ctx, mouseX, mouseY, delta);
    }

    @Override
    public boolean mouseClicked(Click click, boolean doubled) {
        if (click.buttonInfo().button() != 0) {
            return super.mouseClicked(click, doubled);
        }
        int px = width / 2 - 160;
        int py = height / 2 - 80;
        for (int i = 0; i < ROWS; i++) {
            int rowY = py + 44 + i * ROW_H;
            if (click.x() >= px + 12 && click.x() <= px + 308 && click.y() >= rowY && click.y() <= rowY + 22) {
                toggleRow(i);
                ModernConfig.save();
                return true;
            }
        }
        return super.mouseClicked(click, doubled);
    }

    private static String rowLabel(int i) {
        return switch (i) {
            case 0 -> "Toggle Sprint (M)";
            case 1 -> "Auto Sprint (W)";
            default -> "";
        };
    }

    private static String rowDescription(int i) {
        return switch (i) {
            case 0 -> "Mantiene el sprint activo con una tecla, estilo Lunar Client.";
            case 1 -> "Corre automaticamente al ir hacia adelante, sin apretar tecla.";
            default -> "";
        };
    }

    private String rowValue(int i) {
        return switch (i) {
            case 0 -> ModernConfig.toggleSprint ? "ON" : "OFF";
            case 1 -> ModernConfig.toggleSprintLegacy ? "ON" : "OFF";
            default -> "";
        };
    }

    private static int rowColor(int i) {
        boolean on = switch (i) {
            case 0 -> ModernConfig.toggleSprint;
            case 1 -> ModernConfig.toggleSprintLegacy;
            default -> false;
        };
        return on ? 0xFF22CC66 : 0xFFCC4444;
    }

    private static void toggleRow(int i) {
        switch (i) {
            case 0 -> ModernConfig.toggleSprint = !ModernConfig.toggleSprint;
            case 1 -> ModernConfig.toggleSprintLegacy = !ModernConfig.toggleSprintLegacy;
            default -> {}
        }
    }
}
