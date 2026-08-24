package com.paraguacraft.pvp.cosmetics;

import com.paraguacraft.pvp.gui.CustomPauseMenu;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.gui.ScaledResolution;
import net.minecraft.client.gui.inventory.GuiChest;
import net.minecraft.client.gui.inventory.GuiCrafting;
import net.minecraft.client.gui.inventory.GuiFurnace;
import net.minecraft.client.gui.inventory.GuiInventory;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.util.ResourceLocation;
import net.minecraftforge.client.event.GuiScreenEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;

/**
 * Watermark estilo Lunar: solo en pausa, inventario, cofre, horno y mesa de crafteo.
 */
public final class WatermarkRenderer {

    public static final ResourceLocation ICON =
        new ResourceLocation("paraguacraft", "textures/gui/mini_icon.png");
    public static final ResourceLocation BANNER =
        new ResourceLocation("paraguacraft", "textures/gui/watermark.png");

    private static final int BANNER_H = 48;
    private static final float BANNER_ASPECT = 436.0F / 128.0F;
    private static final int PAD = 8;

    @SubscribeEvent
    public void onDrawScreen(GuiScreenEvent.DrawScreenEvent.Post event) {
        Minecraft mc = Minecraft.getMinecraft();
        if (!ModConfig.showWatermark || event.gui == null || mc.theWorld == null) {
            return;
        }
        if (!isContainerScreen(event.gui)) {
            return;
        }
        draw();
    }

    private static boolean isContainerScreen(GuiScreen gui) {
        return gui instanceof GuiInventory
            || gui instanceof GuiChest
            || gui instanceof GuiFurnace
            || gui instanceof GuiCrafting
            || gui instanceof CustomPauseMenu;
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
