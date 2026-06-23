package com.paraguacraft.pvp.gui;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiButton;
import java.awt.Color;

public class FlatButton extends GuiButton {
    
    public boolean isToggled;

    public FlatButton(int buttonId, int x, int y, int widthIn, int heightIn, String buttonText, boolean toggled) {
        super(buttonId, x, y, widthIn, heightIn, buttonText);
        this.isToggled = toggled;
    }

    @Override
    public void drawButton(Minecraft mc, int mouseX, int mouseY) {
        if (this.visible) {
            // Detectar si el puntero del mouse está sobre el botón
            this.hovered = mouseX >= this.xPosition && mouseY >= this.yPosition && mouseX < this.xPosition + this.width && mouseY < this.yPosition + this.height;
            
            // --- PALETA DE COLORES ESTILO WXTER CLIENT ---
            // Fondo oscuro base (Negro profundo con un toque de azul, semi-transparente)
            int fondoBase = new Color(12, 12, 16, 185).getRGB();
            // Fondo cuando el mod está encendido (Cian neón con opacidad suave)
            int fondoActivo = new Color(0, 180, 216, 100).getRGB();
            // Color del borde neón brillante (Cian puro)
            int bordeNeon = new Color(0, 224, 255, 255).getRGB();

            // 1. Dibujar la caja de fondo principal
            if (this.isToggled) {
                Gui.drawRect(this.xPosition, this.yPosition, this.xPosition + this.width, this.yPosition + this.height, fondoActivo);
            } else {
                Gui.drawRect(this.xPosition, this.yPosition, this.xPosition + this.width, this.yPosition + this.height, fondoBase);
            }

            // 2. Si el mouse está encima (Hover) o el mod está activo, inyectamos el borde fino neón
            if (this.hovered || this.isToggled) {
                // Línea Superior
                Gui.drawRect(this.xPosition, this.yPosition, this.xPosition + this.width, this.yPosition + 1, bordeNeon);
                // Línea Inferior
                Gui.drawRect(this.xPosition, this.yPosition + this.height - 1, this.xPosition + this.width, this.yPosition + this.height, bordeNeon);
                // Línea Izquierda
                Gui.drawRect(this.xPosition, this.yPosition, this.xPosition + 1, this.yPosition + this.height, bordeNeon);
                // Línea Derecha
                Gui.drawRect(this.xPosition + this.width - 1, this.yPosition, this.xPosition + this.width, this.yPosition + this.height, bordeNeon);
            }

            // 3. Color del texto: Cian eléctrico si está seleccionado/hovered, o Blanco limpio si está apagado
            int colorTexto = (this.hovered || this.isToggled) ? 0x00E5FF : 0xE0E0E0;

            // Dibujamos la cadena de texto centrada perfectamente en la caja
            this.drawCenteredString(mc.fontRendererObj, this.displayString, this.xPosition + this.width / 2, this.yPosition + (this.height - 8) / 2, colorTexto);
        }
    }
}