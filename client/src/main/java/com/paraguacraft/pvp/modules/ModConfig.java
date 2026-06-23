package com.paraguacraft.pvp.modules;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.util.Properties;
import net.minecraft.client.Minecraft;

public class ModConfig {
    // HUD Originales
    public static boolean showFPS = true;
    public static boolean showPing = true;
    public static boolean showCPS = true;
    public static boolean showKeystrokes = true;
    public static boolean noHurtCam = true;
    
    // Modulos V2.0 (Premium)
    public static boolean showArmor = true;
    public static boolean showPotions = true;
    public static boolean transparentScoreboard = true;
    public static boolean dynamicFov = true; 
    public static int crosshairMode = 0; 
    public static boolean showCoords = true;
    public static boolean toggleSneak = false; 
    public static boolean isSneakingToggled = false; 
    public static boolean showArmorPercentage = true;
    public static boolean borderlessWindow = false;

    // --- NUEVOS MÓDULOS PREMIUM (Corrección Lunar Style) ---
    public static boolean showHeldItem = true; // Reemplaza a 'showHeldEnchants'
    public static boolean showServerHUD = true; // Reemplaza a 'showServerIP'
    public static boolean showCompass = true; // Reemplaza a 'showDirection'
    
    // Coordenadas Iniciales Acomodadas
    public static int fpsX = 5, fpsY = 5;
    public static int pingX = 5, pingY = 15;
    public static int cpsX = 5, cpsY = 25;
    public static int coordsX = 5, coordsY = 35;
    public static int keysX = 5, keysY = 55;
    public static int armorX = 5, armorY = 140; 
    public static int potionX = 150, potionY = 5; // Pociones con fondo translúcido

    // Lugares por defecto para los nuevos módulos
    public static int heldX = 5, heldY = 230; 
    public static int serverX = 5, serverY = 210; 
    public static int compassY = 10; // La brújula se alinea al centro automáticamente, solo definimos la altura

    public static boolean loaded = false;

    public static void save() {
        try {
            File file = new File(Minecraft.getMinecraft().mcDataDir, "paraguacraft_v2.properties");
            Properties props = new Properties();

            // Booleanos
            props.setProperty("showFPS", String.valueOf(showFPS));
            props.setProperty("showPing", String.valueOf(showPing));
            props.setProperty("showCPS", String.valueOf(showCPS));
            props.setProperty("showKeystrokes", String.valueOf(showKeystrokes));
            props.setProperty("noHurtCam", String.valueOf(noHurtCam));
            props.setProperty("showArmor", String.valueOf(showArmor));
            props.setProperty("showPotions", String.valueOf(showPotions));
            props.setProperty("transparentScoreboard", String.valueOf(transparentScoreboard));
            props.setProperty("dynamicFov", String.valueOf(dynamicFov));
            props.setProperty("crosshairMode", String.valueOf(crosshairMode));
            props.setProperty("showCoords", String.valueOf(showCoords));
            props.setProperty("toggleSneak", String.valueOf(toggleSneak));
            props.setProperty("showArmorPercentage", String.valueOf(showArmorPercentage));
            props.setProperty("borderlessWindow", String.valueOf(borderlessWindow));
            
            // Booleanos Nuevos
            props.setProperty("showHeldItem", String.valueOf(showHeldItem));
            props.setProperty("showServerHUD", String.valueOf(showServerHUD));
            props.setProperty("showCompass", String.valueOf(showCompass));

            // Coordenadas
            props.setProperty("fpsX", String.valueOf(fpsX));
            props.setProperty("fpsY", String.valueOf(fpsY));
            props.setProperty("pingX", String.valueOf(pingX));
            props.setProperty("pingY", String.valueOf(pingY));
            props.setProperty("cpsX", String.valueOf(cpsX));
            props.setProperty("cpsY", String.valueOf(cpsY));
            props.setProperty("coordsX", String.valueOf(coordsX));
            props.setProperty("coordsY", String.valueOf(coordsY));
            props.setProperty("keysX", String.valueOf(keysX));
            props.setProperty("keysY", String.valueOf(keysY));
            props.setProperty("armorX", String.valueOf(armorX));
            props.setProperty("armorY", String.valueOf(armorY));
            props.setProperty("potionX", String.valueOf(potionX));
            props.setProperty("potionY", String.valueOf(potionY));
            
            // Coordenadas Nuevas
            props.setProperty("heldX", String.valueOf(heldX));
            props.setProperty("heldY", String.valueOf(heldY));
            props.setProperty("serverX", String.valueOf(serverX));
            props.setProperty("serverY", String.valueOf(serverY));
            props.setProperty("compassY", String.valueOf(compassY));

            props.store(new FileOutputStream(file), "Configuracion de Paraguacraft PvP V2");
        } catch (Exception e) {}
    }

    public static void load() {
        try {
            File file = new File(Minecraft.getMinecraft().mcDataDir, "paraguacraft_v2.properties");
            if (!file.exists()) return;
            Properties props = new Properties();
            props.load(new FileInputStream(file));

            showFPS = Boolean.parseBoolean(props.getProperty("showFPS", String.valueOf(showFPS)));
            showPing = Boolean.parseBoolean(props.getProperty("showPing", String.valueOf(showPing)));
            showCPS = Boolean.parseBoolean(props.getProperty("showCPS", String.valueOf(showCPS)));
            showKeystrokes = Boolean.parseBoolean(props.getProperty("showKeystrokes", String.valueOf(showKeystrokes)));
            noHurtCam = Boolean.parseBoolean(props.getProperty("noHurtCam", String.valueOf(noHurtCam)));
            showArmor = Boolean.parseBoolean(props.getProperty("showArmor", String.valueOf(showArmor)));
            showPotions = Boolean.parseBoolean(props.getProperty("showPotions", String.valueOf(showPotions)));
            transparentScoreboard = Boolean.parseBoolean(props.getProperty("transparentScoreboard", String.valueOf(transparentScoreboard)));
            dynamicFov = Boolean.parseBoolean(props.getProperty("dynamicFov", String.valueOf(dynamicFov)));
            crosshairMode = Integer.parseInt(props.getProperty("crosshairMode", String.valueOf(crosshairMode)));
            showCoords = Boolean.parseBoolean(props.getProperty("showCoords", String.valueOf(showCoords)));
            toggleSneak = Boolean.parseBoolean(props.getProperty("toggleSneak", String.valueOf(toggleSneak)));
            showArmorPercentage = Boolean.parseBoolean(props.getProperty("showArmorPercentage", String.valueOf(showArmorPercentage)));
            borderlessWindow = Boolean.parseBoolean(props.getProperty("borderlessWindow", String.valueOf(borderlessWindow)));
            
            // Booleanos Nuevos
            showHeldItem = Boolean.parseBoolean(props.getProperty("showHeldItem", String.valueOf(showHeldItem)));
            showServerHUD = Boolean.parseBoolean(props.getProperty("showServerHUD", String.valueOf(showServerHUD)));
            showCompass = Boolean.parseBoolean(props.getProperty("showCompass", String.valueOf(showCompass)));

            fpsX = Integer.parseInt(props.getProperty("fpsX", String.valueOf(fpsX)));
            fpsY = Integer.parseInt(props.getProperty("fpsY", String.valueOf(fpsY)));
            pingX = Integer.parseInt(props.getProperty("pingX", String.valueOf(pingX)));
            pingY = Integer.parseInt(props.getProperty("pingY", String.valueOf(pingY)));
            cpsX = Integer.parseInt(props.getProperty("cpsX", String.valueOf(cpsX)));
            cpsY = Integer.parseInt(props.getProperty("cpsY", String.valueOf(cpsY)));
            coordsX = Integer.parseInt(props.getProperty("coordsX", String.valueOf(coordsX)));
            coordsY = Integer.parseInt(props.getProperty("coordsY", String.valueOf(coordsY)));
            keysX = Integer.parseInt(props.getProperty("keysX", String.valueOf(keysX)));
            keysY = Integer.parseInt(props.getProperty("keysY", String.valueOf(keysY)));
            armorX = Integer.parseInt(props.getProperty("armorX", String.valueOf(armorX)));
            armorY = Integer.parseInt(props.getProperty("armorY", String.valueOf(armorY)));
            potionX = Integer.parseInt(props.getProperty("potionX", String.valueOf(potionX)));
            potionY = Integer.parseInt(props.getProperty("potionY", String.valueOf(potionY)));
            
            // Coordenadas Nuevas
            heldX = Integer.parseInt(props.getProperty("heldX", String.valueOf(heldX)));
            heldY = Integer.parseInt(props.getProperty("heldY", String.valueOf(heldY)));
            serverX = Integer.parseInt(props.getProperty("serverX", String.valueOf(serverX)));
            serverY = Integer.parseInt(props.getProperty("serverY", String.valueOf(serverY)));
            compassY = Integer.parseInt(props.getProperty("compassY", String.valueOf(compassY)));

        } catch (Exception e) {}
    }
}