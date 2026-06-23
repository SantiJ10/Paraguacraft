package com.paraguacraft.pvp.gui;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.renderer.texture.DynamicTexture;
import net.minecraftforge.client.event.GuiOpenEvent;
import net.minecraftforge.client.event.GuiScreenEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;

import java.awt.Color;

public class GuiBackgroundHandler extends Gui {

    private final int colorTop = new Color(10, 10, 20, 255).getRGB(); 
    private final int colorBottom = new Color(0, 0, 0, 255).getRGB();   
    
    // Nuestra "tierra falsa" que vivira en la memoria RAM
    private DynamicTexture darkTexture;

    // 1. EL TRUCO: Inyectamos la textura oscura cada vez que abrimos un menu
    @SubscribeEvent
    public void onGuiOpen(GuiOpenEvent event) {
        if (event.gui != null && Minecraft.getMinecraft().getTextureManager() != null) {
            
            // Si todavia no creamos la textura, la fabricamos una sola vez (16x16 pixeles)
            if (darkTexture == null) {
                darkTexture = new DynamicTexture(16, 16);
                int[] pixels = darkTexture.getTextureData();
                for (int i = 0; i < pixels.length; i++) {
                    pixels[i] = colorTop; // Pintamos cada pixel de azul oscuro mate
                }
                darkTexture.updateDynamicTexture();
            }
            
            // Sobrescribimos a la fuerza la textura original de Minecraft.
            // Ahora, cuando las listas busquen "la tierra", van a encontrar nuestro color oscuro.
            Minecraft.getMinecraft().getTextureManager().loadTexture(Gui.optionsBackground, darkTexture);
        }
    }

    // 2. Mantenemos el degradado hermoso para los menus normales (Opciones, Controles, etc.)
    @SubscribeEvent
    public void onGuiDrawBackground(GuiScreenEvent.BackgroundDrawnEvent event) {
        if (Minecraft.getMinecraft().theWorld == null) {
            if (!(event.gui instanceof CustomMainMenu)) {
                this.drawGradientRect(0, 0, event.gui.width, event.gui.height, colorTop, colorBottom);
            }
        }
    }
}