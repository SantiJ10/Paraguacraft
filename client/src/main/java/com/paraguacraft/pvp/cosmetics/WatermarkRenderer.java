package com.paraguacraft.pvp.cosmetics;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.ScaledResolution;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.util.ResourceLocation;
import net.minecraftforge.client.event.GuiScreenEvent;
import net.minecraftforge.client.event.RenderGameOverlayEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;

/**
 * Watermark estilo Lunar: banner (icono + PARAGUACRAFT) en la esquina inferior derecha.
 * En partida se dibuja en el HUD; con un GUI abierto, encima de la pantalla.
 */
public final class WatermarkRenderer {

    public static final ResourceLocation ICON =
        new ResourceLocation("paraguacraft", "textures/gui/mini_icon.png");
    public static final ResourceLocation BANNER =
        new ResourceLocation("paraguacraft", "textures/gui/watermark.png");

    /** Alto en px GUI, comparable al watermark de Lunar Client. */
    private static final int BANNER_H = 48;
    private static final float BANNER_ASPECT = 436.0F / 128.0F;
    private static final int PAD = 8;

    @SubscribeEvent
    public void onHud(RenderGameOverlayEvent.Text event) {
        if (Minecraft.getMinecraft().currentScreen != null) {
            return;
        }
        draw();
    }

    @SubscribeEvent
    public void onDrawScreen(GuiScreenEvent.DrawScreenEvent.Post event) {
        Minecraft mc = Minecraft.getMinecraft();
        if (!ModConfig.showWatermark || event.gui == null || mc.theWorld == null) {
            return;
        }
        draw();
    }

    public static void draw() {
        if (!ModConfig.showWatermark) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.getTextureManager() == null) {
            return;
        }
        ScaledResolution sr = new ScaledResolution(mc);
        int h = BANNER_H;
        int w = Math.round(h * BANNER_ASPECT);
        int x = sr.getScaledWidth() - PAD - w;
        int y = sr.getScaledHeight() - PAD - h;

        GlStateManager.pushMatrix();
        try {
            GlStateManager.disableLighting();
            GlStateManager.disableDepth();
            GlStateManager.enableBlend();
            GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
            GlStateManager.color(1.0F, 1.0F, 1.0F, 0.95F);
            GlStateManager.enableTexture2D();
            mc.getTextureManager().bindTexture(BANNER);
            Gui.drawModalRectWithCustomSizedTexture(x, y, 0, 0, w, h, w, h);
        } finally {
            GlStateManager.enableDepth();
            GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
            GlStateManager.popMatrix();
        }
    }
}
