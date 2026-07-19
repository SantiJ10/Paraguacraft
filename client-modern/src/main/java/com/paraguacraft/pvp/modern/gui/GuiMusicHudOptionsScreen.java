package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.Click;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;

/** Opciones del HUD de musica: portada, transparencia y tamano. */
public class GuiMusicHudOptionsScreen extends ParaguacraftScreen {

    private static final int ROWS = 3;

    public GuiMusicHudOptionsScreen(Screen parent) {
        super(Text.literal("HUD Musica"), parent);
    }

    @Override
    public void render(DrawContext ctx, int mouseX, int mouseY, float delta) {
        renderBackground(ctx, mouseX, mouseY, delta);
        int px = width / 2 - 160;
        int py = height / 2 - 84;
        ctx.fill(px, py, px + 320, py + 168, 0xCC0A0C14);
        ctx.drawText(textRenderer, Text.literal("HUD Musica (Spotify / YouTube)"), px + 16, py + 12, UiTheme.accent(), true);
        for (int i = 0; i < ROWS; i++) {
            int rowY = py + 44 + i * 32;
            ctx.fill(px + 12, rowY, px + 308, rowY + 22, 0x44000000);
            ctx.drawText(textRenderer, Text.literal(rowLabel(i)), px + 20, rowY + 7, UiTheme.TEXT, true);
            String value = rowValue(i);
            ctx.drawText(textRenderer, Text.literal(value), px + 288 - textRenderer.getWidth(value), rowY + 7, rowColor(i), true);
        }
        super.render(ctx, mouseX, mouseY, delta);
    }

    @Override
    public boolean mouseClicked(Click click, boolean doubled) {
        if (click.buttonInfo().button() != 0) {
            return super.mouseClicked(click, doubled);
        }
        int px = width / 2 - 160;
        int py = height / 2 - 84;
        for (int i = 0; i < ROWS; i++) {
            int rowY = py + 44 + i * 32;
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
            case 0 -> "Descargar portada (caratula)";
            case 1 -> "Transparencia del panel";
            case 2 -> "Tamano del HUD";
            default -> "";
        };
    }

    private String rowValue(int i) {
        return switch (i) {
            case 0 -> ModernConfig.showMusicAlbumArt ? "ON" : "OFF";
            case 1 -> ModernConfig.musicHudAlphaLabel();
            case 2 -> ModernConfig.musicHudScaleLabel();
            default -> "";
        };
    }

    private static int rowColor(int i) {
        if (i == 0) {
            return ModernConfig.showMusicAlbumArt ? 0xFF22CC66 : 0xFFCC4444;
        }
        return UiTheme.accent();
    }

    private static void toggleRow(int i) {
        switch (i) {
            case 0 -> ModernConfig.showMusicAlbumArt = !ModernConfig.showMusicAlbumArt;
            case 1 -> ModernConfig.cycleMusicHudAlpha();
            case 2 -> ModernConfig.cycleMusicHudScale();
            default -> {}
        }
    }
}
