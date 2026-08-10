package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.gui.theme.UiTheme;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.texture.DynamicTexture;
import net.minecraftforge.client.event.GuiOpenEvent;
import net.minecraftforge.client.event.GuiScreenEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;

import java.util.Random;

/**
 * Fondo oscuro Paraguacraft en menús (optionsBackground + refuerzo visual).
 * El hover celeste de botones/listas vive en mixins de GuiButton / GuiSlot.
 */
public class GuiBackgroundHandler extends Gui {

    private DynamicTexture darkTexture;

    @SubscribeEvent
    public void onGuiOpen(GuiOpenEvent event) {
        if (event.gui == null) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc == null || mc.getTextureManager() == null) {
            return;
        }
        if (darkTexture == null) {
            darkTexture = new DynamicTexture(16, 16);
            int[] pixels = darkTexture.getTextureData();
            // Navy mate con micro-variación (evita plano puro y se ve “client”)
            Random rnd = new Random(0x50415241L);
            for (int i = 0; i < pixels.length; i++) {
                int v = 8 + rnd.nextInt(6);
                int b = 16 + rnd.nextInt(8);
                // ARGB: opaco, r~v, g~v, b un poco más
                pixels[i] = (0xFF << 24) | (v << 16) | (v << 8) | b;
            }
            darkTexture.updateDynamicTexture();
        }
        // Cuando los menús van a "la tierra" / options_background, usan esta textura oscura.
        mc.getTextureManager().loadTexture(Gui.optionsBackground, darkTexture);
    }

    @SubscribeEvent
    public void onGuiDrawBackground(GuiScreenEvent.BackgroundDrawnEvent event) {
        GuiScreen gui = event.gui;
        if (gui == null || gui instanceof CustomMainMenu) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        // Menús sin mundo: un poco más de oscurecido encima del tile
        if (mc.theWorld == null) {
            GlStateManager.disableDepth();
            this.drawGradientRect(0, 0, gui.width, gui.height, UiTheme.BG_TOP, UiTheme.BG_BOTTOM);
            GlStateManager.enableDepth();
        }
    }
}
