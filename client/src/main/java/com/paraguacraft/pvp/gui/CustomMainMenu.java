package com.paraguacraft.pvp.gui;

import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiButton;
import net.minecraft.client.gui.GuiMultiplayer;
import net.minecraft.client.gui.GuiOptions;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.gui.GuiSelectWorld;
import net.minecraft.client.renderer.GlStateManager;
import org.lwjgl.opengl.GL11;

import java.awt.Color;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;

public class CustomMainMenu extends GuiScreen {

    // Lista que va a guardar todas nuestras estrellas
    private List<Star> stars = new ArrayList<Star>();
    private Random random = new Random();

    @Override
    public void initGui() {
        int startY = this.height / 2;
        int btnWidth = 200;
        int btnHeight = 24;
        int spacing = 28;

        // Limpiamos y creamos los botones (Usando tu FlatButton cian)
        this.buttonList.clear();
        this.buttonList.add(new FlatButton(1, this.width / 2 - btnWidth / 2, startY, btnWidth, btnHeight, "Un Jugador", false));
        this.buttonList.add(new FlatButton(2, this.width / 2 - btnWidth / 2, startY + spacing, btnWidth, btnHeight, "Multijugador", false));
        this.buttonList.add(new FlatButton(0, this.width / 2 - btnWidth / 2, startY + spacing * 2, btnWidth, btnHeight, "Opciones", false));
        this.buttonList.add(new FlatButton(4, this.width / 2 - btnWidth / 2, startY + spacing * 3, btnWidth, btnHeight, "Salir del Juego", false));

        // Generamos el universo: 120 estrellas repartidas por toda tu pantalla
        stars.clear();
        for (int i = 0; i < 120; i++) {
            stars.add(new Star(random.nextInt(this.width), random.nextInt(this.height)));
        }
    }

    @Override
    protected void actionPerformed(GuiButton button) {
        if (button.id == 1) this.mc.displayGuiScreen(new GuiSelectWorld(this));
        if (button.id == 2) this.mc.displayGuiScreen(new GuiMultiplayer(this));
        if (button.id == 0) this.mc.displayGuiScreen(new GuiOptions(this, this.mc.gameSettings));
        if (button.id == 4) this.mc.shutdown();
    }

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        // 1. Dibujar el vacio del espacio (Fondo gris/azul ultra oscuro)
        Gui.drawRect(0, 0, this.width, this.height, new Color(8, 8, 12).getRGB());

        // 2. MAGIA OPENGL: Renderizar las estrellas y sus conexiones
        GlStateManager.pushMatrix();
        GL11.glDisable(GL11.GL_TEXTURE_2D);
        GL11.glEnable(GL11.GL_BLEND);
        GL11.glBlendFunc(GL11.GL_SRC_ALPHA, GL11.GL_ONE_MINUS_SRC_ALPHA);
        GL11.glEnable(GL11.GL_LINE_SMOOTH); // Suavizado para que no se vean pixeladas

        // Dibujar las lineas (Constelaciones)
        GL11.glLineWidth(1.0F);
        GL11.glBegin(GL11.GL_LINES);
        for (int i = 0; i < stars.size(); i++) {
            Star s1 = stars.get(i);
            s1.update(); // Movemos la estrella

            for (int j = i + 1; j < stars.size(); j++) {
                Star s2 = stars.get(j);
                
                // Calculamos la distancia entre dos estrellas
                float dx = s1.x - s2.x;
                float dy = s1.y - s2.y;
                float distance = (dx * dx) + (dy * dy);
                
                // Si estan muy cerca (menos de 60 pixeles de distancia), trazamos una linea cian
                if (distance < 3600) {
                    float alpha = 1.0F - (distance / 3600F); // Mientras mas cerca, mas brillante
                    // Color Cian Neon (R:0, G:229, B:255)
                    GL11.glColor4f(0.0F, 0.898F, 1.0F, alpha * 0.5F); 
                    GL11.glVertex2f(s1.x, s1.y);
                    GL11.glVertex2f(s2.x, s2.y);
                }
            }
        }
        GL11.glEnd();

        // Dibujar los puntos (Las estrellas en si)
        GL11.glPointSize(2.0F); // Tamano de la estrella
        GL11.glBegin(GL11.GL_POINTS);
        for (Star s : stars) {
            GL11.glColor4f(0.0F, 0.898F, 1.0F, 0.8F); // Cian
            GL11.glVertex2f(s.x, s.y);
        }
        GL11.glEnd();

        GL11.glDisable(GL11.GL_BLEND);
        GL11.glEnable(GL11.GL_TEXTURE_2D);
        GlStateManager.popMatrix();

        // 3. Titulo del Launcher con sombras
        GlStateManager.pushMatrix();
        GlStateManager.scale(3.0F, 3.0F, 3.0F);
        this.drawCenteredString(this.fontRendererObj, "PARAGUACRAFT", this.width / 2 / 3, (this.height / 2 - 80) / 3, 0xFFFFFF);
        GlStateManager.popMatrix();

        // 4. Dibujar los botones por encima de todo
        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    // --- CLASE INTERNA DE FISICA DE ESTRELLAS ---
    private class Star {
        float x, y, speedX, speedY;

        public Star(float x, float y) {
            this.x = x;
            this.y = y;
            // Velocidad aleatoria muy suave para cada estrella
            this.speedX = (random.nextFloat() - 0.5F) * 0.5F;
            this.speedY = (random.nextFloat() - 0.5F) * 0.5F;
        }

        public void update() {
            this.x += this.speedX;
            this.y += this.speedY;

            // Si la estrella se sale de la pantalla, la teletransportamos al otro lado
            if (this.x < 0) this.x = width;
            if (this.x > width) this.x = 0;
            if (this.y < 0) this.y = height;
            if (this.y > height) this.y = 0;
        }
    }
}