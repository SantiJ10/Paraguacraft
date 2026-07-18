package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.ParaguacraftPvPModern;
import com.paraguacraft.pvp.modern.gui.theme.MenuTheme;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gl.RenderPipelines;
import net.minecraft.text.Text;
import net.minecraft.util.Identifier;

/** Fondo del menu: tema completo + logo Paraguacraft. */
public final class MenuBackground {

    public static final int LOGO_MAX = 88;
    private static final int LOGO_TEX = 128;

    private static final Identifier LOGO = Identifier.of(
        ParaguacraftPvPModern.MOD_ID,
        "textures/gui/logo.png"
    );

    private MenuBackground() {}

    public static int headerBottom(int screenHeight) {
        int logoSize = logoSize(screenHeight, screenHeight);
        int ly = Math.max(24, screenHeight / 8);
        return ly + logoSize + 46;
    }

    /** @deprecated use {@link #headerBottom(int)} */
    public static int contentTop(int screenHeight) {
        return headerBottom(screenHeight);
    }

    private static int logoSize(int w, int h) {
        int size = Math.min(LOGO_MAX, w / 5);
        return Math.min(size, h / 7);
    }

    public static void draw(Screen screen, DrawContext ctx, int mouseX, int mouseY, float delta) {
        int w = screen.width;
        int h = screen.height;
        MenuTheme theme = MenuTheme.current();

        drawThemeBackground(ctx, w, h, theme);
        if (theme.animatedOverlay) {
            ConstellationBackground.draw(ctx, w, h);
        }

        int logoSize = logoSize(w, h);
        int lx = w / 2 - logoSize / 2;
        int ly = Math.max(24, h / 8);

        MinecraftClient client = MinecraftClient.getInstance();
        if (client.getResourceManager().getResource(LOGO).isPresent()) {
            ctx.drawTexture(RenderPipelines.GUI_TEXTURED, LOGO, lx, ly, 0f, 0f, logoSize, logoSize, LOGO_TEX, LOGO_TEX);
        }

        var tr = client.textRenderer;
        String title = "PARAGUACRAFT";
        ctx.drawText(tr, Text.literal(title), w / 2 - tr.getWidth(title) / 2, ly + logoSize + 8, UiTheme.accent(), true);
        ctx.drawText(
            tr,
            Text.literal("PvP Modern · 1.21.11"),
            w / 2 - tr.getWidth("PvP Modern · 1.21.11") / 2,
            ly + logoSize + 24,
            UiTheme.textDim(),
            true
        );
    }

    private static void drawThemeBackground(DrawContext ctx, int w, int h, MenuTheme theme) {
        if (theme.backgroundTexture != null) {
            MinecraftClient client = MinecraftClient.getInstance();
            if (client.getResourceManager().getResource(theme.backgroundTexture).isPresent()) {
                ctx.drawTexture(RenderPipelines.GUI_TEXTURED, theme.backgroundTexture, 0, 0, 0f, 0f, w, h, 256, 256);
                ctx.fill(0, 0, w, h, theme.backgroundTint);
                return;
            }
        }
        ctx.fillGradient(0, 0, w, h, UiTheme.bgTop(), UiTheme.bgBottom());
        ctx.fill(0, 0, w, h, UiTheme.OVERLAY);
        if (theme.secondaryTint != 0) {
            ctx.fill(0, 0, w, h / 2, theme.secondaryTint);
        }
    }
}
