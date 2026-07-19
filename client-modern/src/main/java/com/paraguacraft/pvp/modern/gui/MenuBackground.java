package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.ParaguacraftPvPModern;
import com.paraguacraft.pvp.modern.gui.theme.MenuTheme;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.multiplayer.ConnectScreen;
import net.minecraft.client.gui.screen.multiplayer.MultiplayerScreen;
import net.minecraft.client.gui.screen.option.OptionsScreen;
import net.minecraft.client.gui.screen.world.SelectWorldScreen;
import net.minecraft.client.gl.RenderPipelines;
import net.minecraft.text.Text;
import net.minecraft.util.Identifier;

/** Fondo del menu: tema completo + logo Paraguacraft (layout 1.8.9). */
public final class MenuBackground {

    private static final Identifier LOGO = Identifier.of(
        ParaguacraftPvPModern.MOD_ID,
        "textures/gui/logo.png"
    );

    private MenuBackground() {}

    public static int headerBottom(int screenHeight) {
        int logoSize = logoSize(screenHeight, screenHeight);
        int ly = logoY(screenHeight, logoSize);
        return ly + logoSize + 58;
    }

    /** @deprecated use {@link #headerBottom(int)} */
    public static int contentTop(int screenHeight) {
        return headerBottom(screenHeight);
    }

    private static int logoSize(int w, int h) {
        return Math.min(w / 5, 96);
    }

    private static int logoY(int h, int logoSize) {
        return h / 5 - logoSize / 2;
    }

    public static void draw(Screen screen, DrawContext ctx, int mouseX, int mouseY, float delta) {
        draw(screen, ctx, mouseX, mouseY, delta, true);
    }

    public static void draw(Screen screen, DrawContext ctx, int mouseX, int mouseY, float delta, boolean withBranding) {
        int w = screen.width;
        int h = screen.height;
        MenuTheme theme = MenuTheme.current();

        drawThemeBackground(ctx, w, h, theme);
        if (theme.animatedOverlay) {
            ConstellationBackground.draw(ctx, w, h);
        }
        if (!withBranding) {
            return;
        }

        int logoSize = logoSize(w, h);
        int lx = w / 2 - logoSize / 2;
        int ly = logoY(h, logoSize);

        MinecraftClient client = MinecraftClient.getInstance();
        if (client.getResourceManager().getResource(LOGO).isPresent()) {
            // Igual que 1.8.9: UV 0-1 sobre toda la textura, evita recortar el icono.
            ctx.drawTexture(RenderPipelines.GUI_TEXTURED, LOGO, lx, ly, 0f, 0f, logoSize, logoSize, logoSize, logoSize);
        }

        var tr = client.textRenderer;
        String title = "PARAGUACRAFT";
        int titleW = tr.getWidth(title);
        var matrices = ctx.getMatrices();
        matrices.pushMatrix();
        matrices.translate(w / 2f, ly + logoSize + 10f);
        matrices.scale(2f, 2f);
        ctx.drawText(tr, Text.literal(title), -titleW / 2, 0, UiTheme.accent(), true);
        matrices.popMatrix();

        ctx.drawText(
            tr,
            Text.literal("PvP Modern · 1.21.11"),
            w / 2 - tr.getWidth("PvP Modern · 1.21.11") / 2,
            ly + logoSize + 48,
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
            int top = theme.secondaryTint;
            int bottom = top & 0x00FFFFFF;
            ctx.fillGradient(0, 0, w, h, top, bottom);
        }
    }

    /** Reemplaza el fondo negro/vanilla en menus fuera del mundo. */
    public static boolean shouldReplace(Screen screen) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client == null || client.world != null) {
            return false;
        }
        if (screen instanceof CustomTitleScreen) {
            return false;
        }
        if (screen instanceof ParaguacraftMultiplayerScreen
            || screen instanceof HypixelQuickPlayScreen
            || screen instanceof ModMenuScreen
            || screen instanceof ThemeSelectScreen
            || screen instanceof PackSelectScreen
            || screen instanceof SkinChangerScreen
            || screen instanceof GuiEditHudScreen) {
            return false;
        }
        return screen instanceof MultiplayerScreen
            || screen instanceof ConnectScreen
            || screen instanceof SelectWorldScreen
            || screen instanceof OptionsScreen
            || screen.getClass().getName().contains("ProgressScreen")
            || screen.getClass().getName().contains("Loading");
    }
}
