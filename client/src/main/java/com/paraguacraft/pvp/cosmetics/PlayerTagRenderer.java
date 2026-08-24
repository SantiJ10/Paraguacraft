package com.paraguacraft.pvp.cosmetics;

import com.paraguacraft.pvp.modules.ModConfig;
import com.paraguacraft.pvp.network.ParaguacraftNetwork;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.inventory.GuiInventory;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.RenderHelper;
import net.minecraft.client.renderer.Tessellator;
import net.minecraft.client.renderer.WorldRenderer;
import net.minecraft.client.renderer.vertex.DefaultVertexFormats;
import net.minecraft.entity.player.EntityPlayer;
import net.minecraft.util.ResourceLocation;

/**
 * Overlay 2D sobre el modelo 3D: nombre + mini-logo + vida.
 * Anclado al centro del {@code GuiInventory.drawEntityOnScreen} vanilla
 * ({@code guiLeft + 51}, {@code guiTop + 75}, scale 30).
 */
public final class PlayerTagRenderer {

    private static final ResourceLocation ICONS = new ResourceLocation("textures/gui/icons.png");
    public static final int HEART = 9;
    /** Vanilla {@code GuiInventory.drawEntityOnScreen} scale. */
    private static final int MODEL_SCALE = 30;
    private static final int MODEL_OFFSET_X = 51;
    private static final int MODEL_FEET_Y = 75;
    private static final float OVERLAY_SCALE = 0.85F;

    /** >0 while {@link GuiInventory#drawEntityOnScreen} is rendering a GUI preview. */
    private static int guiEntityDepth;

    private PlayerTagRenderer() {}

    public static void beginGuiEntityPass() {
        guiEntityDepth++;
    }

    public static void endGuiEntityPass() {
        if (guiEntityDepth > 0) {
            guiEntityDepth--;
        }
    }

    public static boolean isGuiEntityPass() {
        return guiEntityDepth > 0;
    }

    public static void drawInventoryOverlay(int guiLeft, int guiTop, EntityPlayer player) {
        drawOnModel(guiLeft + MODEL_OFFSET_X, guiTop + MODEL_FEET_Y, player);
    }

    /**
     * Preview 3D + overlay for chests, furnaces and the pause menu
     * (screens that do not already draw the player).
     */
    public static void drawPreview(int feetX, int feetY, int mouseX, int mouseY, EntityPlayer player) {
        if (player == null || !ModConfig.showInventoryTags) {
            return;
        }
        GuiInventory.drawEntityOnScreen(
            feetX,
            feetY,
            MODEL_SCALE,
            (float) (feetX - mouseX),
            (float) (feetY - 50 - mouseY),
            player
        );
        drawOnModel(feetX, feetY, player);
        GlStateManager.disableLighting();
        GlStateManager.enableTexture2D();
        GlStateManager.enableAlpha();
        GlStateManager.enableBlend();
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
    }

    public static void drawOnModel(int modelCenterX, int modelFeetY, EntityPlayer player) {
        if (player == null || !ModConfig.showInventoryTags) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        FontRenderer fr = mc.fontRendererObj;
        int modelHeadY = modelFeetY - MODEL_SCALE * 2;
        int nameY = modelHeadY - fr.FONT_HEIGHT - 2;
        String name = player.getName();
        int nameW = fr.getStringWidth(name);
        boolean logo = ModConfig.showNametagLogo && ParaguacraftNetwork.hasLogo(player);

        GlStateManager.pushMatrix();
        try {
            GlStateManager.disableDepth();
            GlStateManager.disableLighting();
            GlStateManager.enableBlend();
            GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
            GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
            GlStateManager.enableTexture2D();
            GlStateManager.translate(modelCenterX, nameY, 300.0F);
            GlStateManager.scale(OVERLAY_SCALE, OVERLAY_SCALE, OVERLAY_SCALE);

            int localNameX = -nameW / 2;
            if (logo) {
                NametagLogoRenderer.drawAt(localNameX - NametagLogoRenderer.LOGO_SIZE - 2, -1);
            }
            fr.drawStringWithShadow(name, localNameX, 0, 0xFFFFFF);
            drawHealthRow(fr, 0, fr.FONT_HEIGHT + 2, player.getHealth());
        } finally {
            GlStateManager.enableDepth();
            GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
            GlStateManager.popMatrix();
            RenderHelper.disableStandardItemLighting();
            GlStateManager.disableLighting();
            GlStateManager.enableTexture2D();
        }
    }

    public static void drawHealthRow(FontRenderer font, int centerX, int y, float health) {
        String hp = String.format("%.1f", Math.max(0.0F, health));
        int w = font.getStringWidth(hp);
        int total = HEART + 2 + w;
        int x = centerX - total / 2;

        GlStateManager.enableTexture2D();
        GlStateManager.enableBlend();
        GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
        Minecraft.getMinecraft().getTextureManager().bindTexture(ICONS);
        textured(x, y, HEART, HEART, 16, 0, HEART, HEART, 256, 256);
        textured(x, y, HEART, HEART, 52, 0, HEART, HEART, 256, 256);
        font.drawStringWithShadow(hp, x + HEART + 2, y, 0xFFFF5555);
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
    }

    private static void textured(
        float x, float y, float w, float h,
        float u, float v, float uw, float vh, float texW, float texH
    ) {
        float u0 = u / texW;
        float v0 = v / texH;
        float u1 = (u + uw) / texW;
        float v1 = (v + vh) / texH;
        Tessellator tess = Tessellator.getInstance();
        WorldRenderer wr = tess.getWorldRenderer();
        wr.begin(7, DefaultVertexFormats.POSITION_TEX);
        wr.pos(x, y + h, 0.0D).tex(u0, v1).endVertex();
        wr.pos(x + w, y + h, 0.0D).tex(u1, v1).endVertex();
        wr.pos(x + w, y, 0.0D).tex(u1, v0).endVertex();
        wr.pos(x, y, 0.0D).tex(u0, v0).endVertex();
        tess.draw();
    }
}
