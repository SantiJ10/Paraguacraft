package com.paraguacraft.pvp.cosmetics;

import com.paraguacraft.pvp.core.GlRenderUtil;
import com.paraguacraft.pvp.network.BadgeProtocol;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.util.ResourceLocation;

/** Dibuja el icono Paraguacraft junto al nametag (Módulo 5). */
public final class NametagLogoRenderer {

    private static final ResourceLocation LOGO = new ResourceLocation("paraguacraft", "textures/gui/tightfault.png");
    public static final int LOGO_SIZE = 9;

    private NametagLogoRenderer() {}

    public static void drawLeftOfName(FontRenderer font, String name) {
        drawLeftOfName(font, name, BadgeProtocol.BADGE_PARAGUACRAFT);
    }

    public static void drawLeftOfName(FontRenderer font, String name, byte badgeId) {
        int half = font.getStringWidth(name) / 2;
        int x = -half - LOGO_SIZE - 2;
        int y = (font.FONT_HEIGHT - LOGO_SIZE) / 2 - 1;

        GlStateManager.pushMatrix();
        try {
            GlStateManager.enableTexture2D();
            GlStateManager.enableBlend();
            GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
            if (badgeId == BadgeProtocol.BADGE_STAFF) {
                GlStateManager.color(1.0F, 0.85F, 0.2F, 1.0F);
            } else {
                GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
            }
            Minecraft.getMinecraft().getTextureManager().bindTexture(LOGO);
            Gui.drawModalRectWithCustomSizedTexture(x, y, 0, 0, LOGO_SIZE, LOGO_SIZE, LOGO_SIZE, LOGO_SIZE);
        } finally {
            GlRenderUtil.rebindNametagFont();
            GlStateManager.popMatrix();
        }
    }

    /** Dibuja texto extra (ej. ping) a la derecha del nombre, sin alterar el centrado del nametag. */
    public static void drawRightOfName(FontRenderer font, String name, String text, int color) {
        if (text == null || text.isEmpty()) {
            return;
        }
        int x = font.getStringWidth(name) / 2 + 2;
        GlStateManager.pushMatrix();
        try {
            GlStateManager.enableTexture2D();
            GlStateManager.enableBlend();
            GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
            font.drawString(text, x, 0, color);
        } finally {
            GlRenderUtil.rebindNametagFont();
            GlStateManager.popMatrix();
        }
    }
}
