package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.Click;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;

/** Insignias (servidores Fabric/Paper con Paraguacraft Badges) y ping rival en nametags. */
public class GuiBadgesPingOptionsScreen extends ParaguacraftScreen {

    private static final int ROWS = 3;
    private static final int ROW_H = 44;

    public GuiBadgesPingOptionsScreen(Screen parent) {
        super(Text.literal("Insignias y Ping"), parent);
    }

    @Override
    public void render(DrawContext ctx, int mouseX, int mouseY, float delta) {
        renderBackground(ctx, mouseX, mouseY, delta);
        int px = width / 2 - 160;
        int py = height / 2 - 105;
        ctx.fill(px, py, px + 320, py + 210, 0xCC0A0C14);
        ctx.drawText(textRenderer, Text.literal("Insignias y Ping"), px + 16, py + 12, UiTheme.accent(), true);
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
        int py = height / 2 - 105;
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
            case 0 -> "Insignia propia";
            case 1 -> "Insignias de rivales";
            case 2 -> "Ping rival en nametag";
            default -> "";
        };
    }

    private static String rowDescription(int i) {
        return switch (i) {
            case 0 -> "Muestra tu insignia (Paraguacraft Badges) sobre tu propio nombre.";
            case 1 -> "Muestra la insignia de otros jugadores sobre su nombre.";
            case 2 -> "Muestra el ping (ms) de otros jugadores junto a su nombre.";
            default -> "";
        };
    }

    private String rowValue(int i) {
        return switch (i) {
            case 0 -> ModernConfig.showNametagLogo ? "ON" : "OFF";
            case 1 -> ModernConfig.showNametagLogoOthers ? "ON" : "OFF";
            case 2 -> ModernConfig.showOpponentPing ? "ON" : "OFF";
            default -> "";
        };
    }

    private static int rowColor(int i) {
        boolean on = switch (i) {
            case 0 -> ModernConfig.showNametagLogo;
            case 1 -> ModernConfig.showNametagLogoOthers;
            case 2 -> ModernConfig.showOpponentPing;
            default -> false;
        };
        return on ? 0xFF22CC66 : 0xFFCC4444;
    }

    private static void toggleRow(int i) {
        switch (i) {
            case 0 -> ModernConfig.showNametagLogo = !ModernConfig.showNametagLogo;
            case 1 -> ModernConfig.showNametagLogoOthers = !ModernConfig.showNametagLogoOthers;
            case 2 -> ModernConfig.showOpponentPing = !ModernConfig.showOpponentPing;
            default -> {}
        }
    }
}
