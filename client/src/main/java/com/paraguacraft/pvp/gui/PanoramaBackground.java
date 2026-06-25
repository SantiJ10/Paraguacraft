package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.gui.theme.UiTheme;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.util.ResourceLocation;

/**
 * Fondo oscuro + logo Paraguacraft.
 */
public final class PanoramaBackground {

    private static final ResourceLocation LOGO = new ResourceLocation("paraguacraft", "textures/gui/logo.png");

    private PanoramaBackground() {}

    private static void fillBackground(int w, int h) {
        Gui.drawRect(0, 0, w, h, UiTheme.BG_TOP);
        Gui.drawRect(0, h / 2, w, h, UiTheme.BG_BOTTOM);
        Gui.drawRect(0, 0, w, h, UiTheme.OVERLAY);
    }

    public static void draw(GuiScreen screen, float partialTicks) {
        int w = screen.width;
        int h = screen.height;
        FontRenderer fr = screen.mc.fontRendererObj;

        fillBackground(w, h);
        ConstellationBackground.draw(w, h, partialTicks);

        GlStateManager.enableBlend();
        GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
        GlStateManager.color(1f, 1f, 1f, 1f);

        int logoSize = Math.min(w / 5, 96);
        int lx = w / 2 - logoSize / 2;
        int ly = h / 5 - logoSize / 2;
        screen.mc.getTextureManager().bindTexture(LOGO);
        Gui.drawModalRectWithCustomSizedTexture(lx, ly, 0, 0, logoSize, logoSize, logoSize, logoSize);

        GlStateManager.color(1f, 1f, 1f, 1f);

        String title = "PARAGUACRAFT";
        int titleW = fr.getStringWidth(title);
        GlStateManager.pushMatrix();
        GlStateManager.translate(w / 2f, ly + logoSize + 10, 0);
        GlStateManager.scale(2.0f, 2.0f, 1f);
        fr.drawStringWithShadow(title, -titleW / 2, 0, UiTheme.ACCENT);
        GlStateManager.popMatrix();

        String subtitle = "Client V2";
        fr.drawStringWithShadow(
            subtitle,
            w / 2 - fr.getStringWidth(subtitle) / 2,
            ly + logoSize + 48,
            UiTheme.TEXT_DIM
        );
    }

    public static void drawLoading(GuiScreen screen, String headline, String subline) {
        int w = screen.width;
        int h = screen.height;
        FontRenderer fr = screen.mc.fontRendererObj;
        fillBackground(w, h);

        GlStateManager.enableBlend();
        GlStateManager.color(1f, 1f, 1f, 1f);
        int logoSize = 48;
        screen.mc.getTextureManager().bindTexture(LOGO);
        Gui.drawModalRectWithCustomSizedTexture(w / 2 - logoSize / 2, h / 2 - 70, 0, 0, logoSize, logoSize, logoSize, logoSize);
        GlStateManager.color(1f, 1f, 1f, 1f);

        fr.drawStringWithShadow(headline, w / 2 - fr.getStringWidth(headline) / 2, h / 2 - 8, UiTheme.TEXT);
        if (subline != null && !subline.isEmpty()) {
            fr.drawStringWithShadow(subline, w / 2 - fr.getStringWidth(subline) / 2, h / 2 + 14, UiTheme.TEXT_DIM);
        }
    }
}
