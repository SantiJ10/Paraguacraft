package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.ParaguacraftPvPModern;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;

/** Fondo del menú: gradiente + constelación + logo Paraguacraft. */
public final class MenuBackground {

    private MenuBackground() {}

    public static void draw(Screen screen, DrawContext ctx, int mouseX, int mouseY, float delta) {
        int w = screen.width;
        int h = screen.height;
        ctx.fillGradient(0, 0, w, h, UiTheme.bgTop(), UiTheme.bgBottom());
        ctx.fill(0, 0, w, h, UiTheme.OVERLAY);
        ConstellationBackground.draw(ctx, w, h);

        int logoSize = Math.min(w / 5, 96);
        int lx = w / 2 - logoSize / 2;
        int ly = h / 5 - logoSize / 2;
        ctx.fill(lx, ly, lx + logoSize, ly + logoSize, UiTheme.accent() & 0x44FFFFFF);
        ctx.fill(lx + 4, ly + 4, lx + logoSize - 4, ly + logoSize - 4, UiTheme.BTN_BG);

        var tr = MinecraftClient.getInstance().textRenderer;
        String title = "PARAGUACRAFT";
        ctx.drawText(tr, Text.literal(title), w / 2 - tr.getWidth(title), ly + logoSize + 10, UiTheme.accent(), true);

        String subtitle = "PvP Modern · 1.21.11";
        ctx.drawText(
            tr,
            Text.literal(subtitle),
            w / 2 - tr.getWidth(subtitle) / 2,
            ly + logoSize + 28,
            UiTheme.textDim(),
            true
        );
    }
}
