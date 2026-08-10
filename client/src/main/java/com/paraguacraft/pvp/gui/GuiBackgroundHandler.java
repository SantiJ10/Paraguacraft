package com.paraguacraft.pvp.gui;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraftforge.client.event.GuiScreenEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;

import java.awt.Color;

/**
 * Fondo de menús sin mundo (y refuerzo sutil en menús in-game).
 * Ya no se sobrescribe {@link Gui#optionsBackground} con un DynamicTexture plano:
 * eso dejaba botones y listas sobre negro mate y se veía “sin textura”.
 * El pack oficial aporta {@code options_background.png} con detalle oscuro.
 */
public class GuiBackgroundHandler extends Gui {

    private final int colorTop = new Color(14, 16, 28, 220).getRGB();
    private final int colorBottom = new Color(4, 4, 10, 235).getRGB();

    @SubscribeEvent
    public void onGuiDrawBackground(GuiScreenEvent.BackgroundDrawnEvent event) {
        GuiScreen gui = event.gui;
        if (gui == null) {
            return;
        }
        // Menú principal custom ya pinta su propio look
        if (gui instanceof CustomMainMenu) {
            return;
        }

        Minecraft mc = Minecraft.getMinecraft();
        // Solo reforzar degradado cuando no hay mundo (pantallas puras de menú)
        if (mc.theWorld == null) {
            GlStateManager.disableDepth();
            this.drawGradientRect(0, 0, gui.width, gui.height, colorTop, colorBottom);
            GlStateManager.enableDepth();
        }
    }
}
