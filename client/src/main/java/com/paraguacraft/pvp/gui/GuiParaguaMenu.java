package com.paraguacraft.pvp.gui;

import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.resources.I18n; // Importante para las traducciones
import com.paraguacraft.pvp.modules.ModConfig;
import org.lwjgl.opengl.Display;
import org.lwjgl.opengl.DisplayMode;
import java.io.IOException;

public class GuiParaguaMenu extends GuiScreen {

    private int sidebarWidth = 140;
    private int topbarHeight = 50;
    
    // Categorías (I18n traduce "paraguacraft.cat.all" -> "Todos", "paraguacraft.cat.hud" -> "HUD", etc.)
    private String[] categories = {"all", "new", "hud", "pvp", "mechanics", "server"};
    private int selectedCategory = 0; 

    private class ModEntry {
        int id;
        String langKey; // Clave para la traducción native (ej: "paraguacraft.mod.cps")
        int categoryIndex; 

        ModEntry(int id, String langKey, int categoryIndex) {
            this.id = id;
            this.langKey = langKey;
            this.categoryIndex = categoryIndex;
        }
    }

    // --- MAPEO DE MODS ACTUALIZADOS CON SUS CATEGORIAS Y LLAVES ---
    private ModEntry[] allMods = {
        new ModEntry(0, "fps", 2),             
        new ModEntry(1, "ping", 2),            
        new ModEntry(2, "cps", 2),             
        new ModEntry(3, "keystrokes", 2),      
        new ModEntry(4, "nohurtcam", 3),       
        new ModEntry(5, "coordinates", 2),     
        new ModEntry(6, "armorhud", 2),       
        new ModEntry(7, "armorpercent", 2),         
        new ModEntry(8, "potionshud", 2),     
        new ModEntry(9, "clearscore", 3),     
        new ModEntry(10, "togglesneak", 4),   
        new ModEntry(11, "dynamicfov", 4),    
        new ModEntry(12, "helditemmod", 2),   // <--- CORREGIDO (Ex HeldEnchants)
        new ModEntry(13, "borderless", 4),
        new ModEntry(14, "serverhud", 2),     // <--- CORREGIDO (Ex ServerIP)
        new ModEntry(15, "compasshud", 2)     // <--- CORREGIDO (Ex Direction)
    };

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        this.drawRect(0, 0, this.width, this.height, 0xAA000000);
        this.drawRect(0, 0, sidebarWidth, this.height, 0xCC111111);
        
        GlStateManager.pushMatrix();
        GlStateManager.scale(1.5f, 1.5f, 1.5f);
        this.drawString(this.fontRendererObj, "\u00A7lPARAGUACRAFT", 10, 15, 0x00E5FF);
        GlStateManager.popMatrix();
        this.drawString(this.fontRendererObj, "v2.0 - Client", 15, 38, 0x666666);

        int catY = 70;
        for (int i = 0; i < categories.length; i++) {
            boolean isSelected = (i == selectedCategory);
            boolean isHovered = mouseX >= 0 && mouseX <= sidebarWidth && mouseY >= catY && mouseY <= catY + 25;
            
            if (isSelected) {
                Gui.drawRect(0, catY, sidebarWidth, catY + 25, 0x4400E5FF); 
                Gui.drawRect(0, catY, 3, catY + 25, 0xFF00E5FF); 
            } else if (isHovered) {
                Gui.drawRect(0, catY, sidebarWidth, catY + 25, 0x22FFFFFF); 
            }
            
            // TRADUCCIÓN DE CATEGORÍAS
            String catName = I18n.format("paraguacraft.cat." + categories[i]);
            if (catName.startsWith("paraguacraft.cat.")) catName = categories[i].toUpperCase();

            this.drawString(this.fontRendererObj, catName, 20, catY + 8, isSelected ? 0xFFFFFF : 0xAAAAAA);
            catY += 25;
        }

        this.drawRect(sidebarWidth, 0, this.width, topbarHeight, 0x88111111);
        Gui.drawRect(sidebarWidth + 20, 12, sidebarWidth + 250, 38, 0xAA000000);
        this.drawString(this.fontRendererObj, "\u00A77> Search mods...", sidebarWidth + 30, 20, 0xFFFFFF); 

        int startX = sidebarWidth + 20;
        int startY = topbarHeight + 20;
        int cardW = 160;
        int cardH = 50;
        int gap = 15;
        int xOffset = 0;
        int yOffset = 0;

        for (ModEntry mod : allMods) {
            if (selectedCategory != 0 && mod.categoryIndex != selectedCategory) continue;

            int currentX = startX + (cardW + gap) * xOffset;
            int currentY = startY + (cardH + gap) * yOffset;

            if (currentX + cardW > this.width - 20) {
                xOffset = 0;
                yOffset++;
                currentX = startX;
                currentY = startY + (cardH + gap) * yOffset;
            }

            // Traducimos el nombre del mod
            String modName = I18n.format("paraguacraft.mod." + mod.langKey);
            if (modName.startsWith("paraguacraft.mod.")) modName = mod.langKey; // Fallback

            drawModCard(currentX, currentY, cardW, cardH, modName, getModState(mod.id), mouseX, mouseY);
            xOffset++;
        }
        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    private void drawModCard(int x, int y, int w, int h, String name, boolean state, int mouseX, int mouseY) {
        boolean hovered = mouseX >= x && mouseX <= x + w && mouseY >= y && mouseY <= y + h;
        
        int bgColor = hovered ? 0xCC222222 : 0x99111111;
        Gui.drawRect(x, y, x + w, y + h, bgColor);
        Gui.drawRect(x, y, x + w, y + 1, 0x33FFFFFF); 
        Gui.drawRect(x, y + h - 1, x + w, y + h, 0x33FFFFFF);
        Gui.drawRect(x + 10, y + 10, x + 40, y + 40, 0x4400E5FF);
        this.drawString(this.fontRendererObj, String.valueOf(name.charAt(0)), x + 21, y + 21, 0x00E5FF);
        this.drawString(this.fontRendererObj, name, x + 50, y + 20, state ? 0xFFFFFF : 0xAAAAAA);

        int toggleW = 28;
        int toggleH = 14;
        int toggleX = x + w - toggleW - 10;
        int toggleY = y + (h / 2) - (toggleH / 2);

        Gui.drawRect(toggleX, toggleY, toggleX + toggleW, toggleY + toggleH, state ? 0xFF00E5FF : 0xFF444444);
        if (state) {
            Gui.drawRect(toggleX + toggleW - 12, toggleY + 2, toggleX + toggleW - 2, toggleY + toggleH - 2, 0xFFFFFF);
        } else {
            Gui.drawRect(toggleX + 2, toggleY + 2, toggleX + 12, toggleY + toggleH - 2, 0xFFFFFF);
        }
    }

    @Override
    protected void mouseClicked(int mouseX, int mouseY, int mouseButton) throws IOException {
        if (mouseButton != 0) return; 

        int catY = 70;
        for (int i = 0; i < categories.length; i++) {
            if (mouseX >= 0 && mouseX <= sidebarWidth && mouseY >= catY && mouseY <= catY + 25) {
                selectedCategory = i; 
                return;
            }
            catY += 25;
        }

        int startX = sidebarWidth + 20;
        int startY = topbarHeight + 20;
        int cardW = 160;
        int cardH = 50;
        int gap = 15;
        int xOffset = 0;
        int yOffset = 0;

        for (ModEntry mod : allMods) {
            if (selectedCategory != 0 && mod.categoryIndex != selectedCategory) continue;

            int currentX = startX + (cardW + gap) * xOffset;
            int currentY = startY + (cardH + gap) * yOffset;

            if (currentX + cardW > this.width - 20) {
                xOffset = 0;
                yOffset++;
                currentX = startX;
                currentY = startY + (cardH + gap) * yOffset;
            }

            if (mouseX >= currentX && mouseX <= currentX + cardW && mouseY >= currentY && mouseY <= currentY + cardH) {
                toggleMod(mod.id);
                ModConfig.save(); 
                return;
            }
            xOffset++;
        }
    }

    private boolean getModState(int id) {
        switch (id) {
            case 0: return ModConfig.showFPS;
            case 1: return ModConfig.showPing;
            case 2: return ModConfig.showCPS;
            case 3: return ModConfig.showKeystrokes;
            case 4: return ModConfig.noHurtCam;
            case 5: return ModConfig.showCoords;
            case 6: return ModConfig.showArmor;
            case 7: return ModConfig.showArmorPercentage;
            case 8: return ModConfig.showPotions;
            case 9: return ModConfig.transparentScoreboard;
            case 10: return ModConfig.toggleSneak;
            case 11: return ModConfig.dynamicFov;
            case 12: return ModConfig.showHeldItem;
            case 13: return ModConfig.borderlessWindow;
            case 14: return ModConfig.showServerHUD;
            case 15: return ModConfig.showCompass;
            default: return false;
        }
    }

    private void toggleMod(int id) {
        switch (id) {
            case 0: ModConfig.showFPS = !ModConfig.showFPS; break;
            case 1: ModConfig.showPing = !ModConfig.showPing; break;
            case 2: ModConfig.showCPS = !ModConfig.showCPS; break;
            case 3: ModConfig.showKeystrokes = !ModConfig.showKeystrokes; break;
            case 4: ModConfig.noHurtCam = !ModConfig.noHurtCam; break;
            case 5: ModConfig.showCoords = !ModConfig.showCoords; break;
            case 6: ModConfig.showArmor = !ModConfig.showArmor; break;
            case 7: ModConfig.showArmorPercentage = !ModConfig.showArmorPercentage; break;
            case 8: ModConfig.showPotions = !ModConfig.showPotions; break;
            case 9: ModConfig.transparentScoreboard = !ModConfig.transparentScoreboard; break;
            case 10: ModConfig.toggleSneak = !ModConfig.toggleSneak; ModConfig.isSneakingToggled = false; break;
            case 11: ModConfig.dynamicFov = !ModConfig.dynamicFov; break;
            case 12: ModConfig.showHeldItem = !ModConfig.showHeldItem; break;
            case 14: ModConfig.showServerHUD = !ModConfig.showServerHUD; break;
            case 15: ModConfig.showCompass = !ModConfig.showCompass; break;
            case 13: 
                ModConfig.borderlessWindow = !ModConfig.borderlessWindow; 
                try {
                    if (ModConfig.borderlessWindow) {
                        System.setProperty("org.lwjgl.opengl.Window.undecorated", "true");
                        Display.setDisplayMode(Display.getDesktopDisplayMode());
                        Display.setLocation(0, 0);
                        net.minecraft.client.Minecraft.getMinecraft().gameSettings.fullScreen = false;
                    } else {
                        System.setProperty("org.lwjgl.opengl.Window.undecorated", "false");
                        Display.setDisplayMode(new DisplayMode(854, 480));
                        Display.setResizable(true); 
                    }
                } catch (Exception e) {}
                break;
        }
    }

    @Override
    public boolean doesGuiPauseGame() { return false; }
}