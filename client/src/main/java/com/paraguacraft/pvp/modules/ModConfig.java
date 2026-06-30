package com.paraguacraft.pvp.modules;

import com.paraguacraft.pvp.core.PerformanceConfig;
import com.paraguacraft.pvp.core.OptifinePreset;
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
    public static boolean transparentScoreboard = true; // legacy read only
    public static boolean scoreboardEnabled = true;
    public static boolean scoreboardTransparentBg = false;
    public static boolean scoreboardHideRedNumbers = false;
    public static boolean scoreboardHideStats = false;
    public static boolean dynamicFov = true; 
    public static int crosshairMode = 0; 
    public static boolean showCoords = true;
    public static boolean toggleSneak = false; 
    public static boolean isSneakingToggled = false; 
    public static boolean showArmorPercentage = true;
    public static boolean toggleSprintActive = true;
    public static boolean fullbrightActive = true;

    public static int keyMenu = org.lwjgl.input.Keyboard.KEY_RSHIFT;
    public static int keyEditHud = org.lwjgl.input.Keyboard.KEY_RCONTROL;
    public static int keyToggleSprint = org.lwjgl.input.Keyboard.KEY_V;
    public static int keyFullbright = org.lwjgl.input.Keyboard.KEY_G;
    public static int keyFreelook = org.lwjgl.input.Keyboard.KEY_LMENU;
    public static int keyQuickPlay = org.lwjgl.input.Keyboard.KEY_GRAVE;

    // --- Mods PvP / Hypixel ---
    public static boolean lowFire = true;
    public static boolean coloredBeds = true;
    public static boolean showOpponentPing = false;
    public static boolean chatTriggers = true;
    public static boolean freelookEnabled = true;
    public static boolean reachDisplay = true;
    public static boolean comboCounter = true;
    public static boolean itemPhysics = true;
    public static boolean hideTitles = true;
    public static int reachDisplayX = 5;
    public static int reachDisplayY = 45;
    public static int comboDisplayX = 5;
    public static int comboDisplayY = 55;

    // --- NUEVOS MÓDULOS PREMIUM (Corrección Lunar Style) ---
    public static boolean showHeldItem = true; // Reemplaza a 'showHeldEnchants'
    public static boolean showServerHUD = true; // Reemplaza a 'showServerIP'
    public static boolean showCompass = true; // Reemplaza a 'showDirection'
    public static boolean showNametagLogo = true;
    public static boolean showNametagLogoOthers = true;

    // Módulo 4 — IPC + HUDs avanzados
    public static boolean showHardwareHud = true;
    public static boolean showMusicHud = true;
    public static boolean showMusicAlbumArt = true;
    public static int musicHudAlpha = 255;
    public static boolean showTntCountdown = true;
    public static boolean showBedwarsResources = true;
    public static boolean bwResTransparentBg = false;
    public static boolean forceItem3d = true;
    public static int overlayHudX = 5;
    public static int overlayHudY = 260;
    public static int overlayHudW = 118;
    public static int bwResX = 5;
    public static int bwResY = 320;
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

    private static final int[] MUSIC_ALPHA_PRESETS = {255, 192, 128, 64};

    public static boolean loaded = false;

    public static void cycleMusicHudAlpha() {
        int idx = 0;
        for (int i = 0; i < MUSIC_ALPHA_PRESETS.length; i++) {
            if (musicHudAlpha == MUSIC_ALPHA_PRESETS[i]) {
                idx = (i + 1) % MUSIC_ALPHA_PRESETS.length;
                break;
            }
        }
        musicHudAlpha = MUSIC_ALPHA_PRESETS[idx];
    }

    public static String musicHudAlphaLabel() {
        int pct = Math.round(musicHudAlpha / 255f * 100f);
        return pct + "%";
    }

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
            props.setProperty("scoreboardEnabled", String.valueOf(scoreboardEnabled));
            props.setProperty("scoreboardTransparentBg", String.valueOf(scoreboardTransparentBg));
            props.setProperty("scoreboardHideRedNumbers", String.valueOf(scoreboardHideRedNumbers));
            props.setProperty("scoreboardHideStats", String.valueOf(scoreboardHideStats));
            props.setProperty("transparentScoreboard", String.valueOf(transparentScoreboard));
            props.setProperty("dynamicFov", String.valueOf(dynamicFov));
            props.setProperty("crosshairMode", String.valueOf(crosshairMode));
            props.setProperty("showCoords", String.valueOf(showCoords));
            props.setProperty("toggleSneak", String.valueOf(toggleSneak));
            props.setProperty("showArmorPercentage", String.valueOf(showArmorPercentage));
            props.setProperty("toggleSprintActive", String.valueOf(toggleSprintActive));
            props.setProperty("fullbrightActive", String.valueOf(fullbrightActive));
            props.setProperty("keyMenu", String.valueOf(keyMenu));
            props.setProperty("keyEditHud", String.valueOf(keyEditHud));
            props.setProperty("keyToggleSprint", String.valueOf(keyToggleSprint));
            props.setProperty("keyFullbright", String.valueOf(keyFullbright));
            props.setProperty("keyFreelook", String.valueOf(keyFreelook));
            props.setProperty("keyQuickPlay", String.valueOf(keyQuickPlay));
            props.setProperty("lowFire", String.valueOf(lowFire));
            props.setProperty("coloredBeds", String.valueOf(coloredBeds));
            props.setProperty("showOpponentPing", String.valueOf(showOpponentPing));
            props.setProperty("chatTriggers", String.valueOf(chatTriggers));
            props.setProperty("freelookEnabled", String.valueOf(freelookEnabled));
            props.setProperty("reachDisplay", String.valueOf(reachDisplay));
            props.setProperty("comboCounter", String.valueOf(comboCounter));
            props.setProperty("itemPhysics", String.valueOf(itemPhysics));
            props.setProperty("hideTitles", String.valueOf(hideTitles));
            props.setProperty("reachDisplayX", String.valueOf(reachDisplayX));
            props.setProperty("reachDisplayY", String.valueOf(reachDisplayY));
            props.setProperty("comboDisplayX", String.valueOf(comboDisplayX));
            props.setProperty("comboDisplayY", String.valueOf(comboDisplayY));

            // Booleanos Nuevos
            props.setProperty("showHeldItem", String.valueOf(showHeldItem));
            props.setProperty("showServerHUD", String.valueOf(showServerHUD));
            props.setProperty("showCompass", String.valueOf(showCompass));
            props.setProperty("showNametagLogo", String.valueOf(showNametagLogo));
            props.setProperty("showNametagLogoOthers", String.valueOf(showNametagLogoOthers));
            props.setProperty("showHardwareHud", String.valueOf(showHardwareHud));
            props.setProperty("showMusicHud", String.valueOf(showMusicHud));
            props.setProperty("showMusicAlbumArt", String.valueOf(showMusicAlbumArt));
            props.setProperty("musicHudAlpha", String.valueOf(musicHudAlpha));
            props.setProperty("showTntCountdown", String.valueOf(showTntCountdown));
            props.setProperty("showBedwarsResources", String.valueOf(showBedwarsResources));
            props.setProperty("bwResTransparentBg", String.valueOf(bwResTransparentBg));
            props.setProperty("forceItem3d", String.valueOf(forceItem3d));
            props.setProperty("overlayHudX", String.valueOf(overlayHudX));
            props.setProperty("overlayHudY", String.valueOf(overlayHudY));
            props.setProperty("overlayHudW", String.valueOf(overlayHudW));
            props.setProperty("bwResX", String.valueOf(bwResX));
            props.setProperty("bwResY", String.valueOf(bwResY));
            props.setProperty("boostFps", String.valueOf(PerformanceConfig.boostFps));
            props.setProperty("oldAnimations", String.valueOf(PerformanceConfig.oldAnimations));
            PerformanceConfig.saveToProperties(props);
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
            if (props.containsKey("transparentScoreboard") && !props.containsKey("scoreboardEnabled")) {
                boolean legacy = Boolean.parseBoolean(props.getProperty("transparentScoreboard"));
                scoreboardTransparentBg = legacy;
                scoreboardHideRedNumbers = legacy;
            }
            scoreboardEnabled = Boolean.parseBoolean(props.getProperty("scoreboardEnabled", String.valueOf(scoreboardEnabled)));
            scoreboardTransparentBg = Boolean.parseBoolean(props.getProperty("scoreboardTransparentBg", String.valueOf(scoreboardTransparentBg)));
            scoreboardHideRedNumbers = Boolean.parseBoolean(props.getProperty("scoreboardHideRedNumbers", String.valueOf(scoreboardHideRedNumbers)));
            scoreboardHideStats = Boolean.parseBoolean(props.getProperty("scoreboardHideStats", String.valueOf(scoreboardHideStats)));
            transparentScoreboard = Boolean.parseBoolean(props.getProperty("transparentScoreboard", String.valueOf(transparentScoreboard)));
            dynamicFov = Boolean.parseBoolean(props.getProperty("dynamicFov", String.valueOf(dynamicFov)));
            crosshairMode = Integer.parseInt(props.getProperty("crosshairMode", String.valueOf(crosshairMode)));
            showCoords = Boolean.parseBoolean(props.getProperty("showCoords", String.valueOf(showCoords)));
            toggleSneak = Boolean.parseBoolean(props.getProperty("toggleSneak", String.valueOf(toggleSneak)));
            showArmorPercentage = Boolean.parseBoolean(props.getProperty("showArmorPercentage", String.valueOf(showArmorPercentage)));
            toggleSprintActive = Boolean.parseBoolean(props.getProperty("toggleSprintActive", String.valueOf(toggleSprintActive)));
            fullbrightActive = Boolean.parseBoolean(props.getProperty("fullbrightActive", String.valueOf(fullbrightActive)));
            keyMenu = Integer.parseInt(props.getProperty("keyMenu", String.valueOf(keyMenu)));
            keyEditHud = Integer.parseInt(props.getProperty("keyEditHud", String.valueOf(keyEditHud)));
            keyToggleSprint = Integer.parseInt(props.getProperty("keyToggleSprint", String.valueOf(keyToggleSprint)));
            keyFullbright = Integer.parseInt(props.getProperty("keyFullbright", String.valueOf(keyFullbright)));
            keyFreelook = Integer.parseInt(props.getProperty("keyFreelook", String.valueOf(keyFreelook)));
            keyQuickPlay = Integer.parseInt(props.getProperty("keyQuickPlay", String.valueOf(keyQuickPlay)));
            lowFire = Boolean.parseBoolean(props.getProperty("lowFire", String.valueOf(lowFire)));
            coloredBeds = Boolean.parseBoolean(props.getProperty("coloredBeds", String.valueOf(coloredBeds)));
            showOpponentPing = Boolean.parseBoolean(props.getProperty("showOpponentPing", String.valueOf(showOpponentPing)));
            chatTriggers = Boolean.parseBoolean(props.getProperty("chatTriggers", String.valueOf(chatTriggers)));
            freelookEnabled = Boolean.parseBoolean(props.getProperty("freelookEnabled", String.valueOf(freelookEnabled)));
            reachDisplay = Boolean.parseBoolean(props.getProperty("reachDisplay", String.valueOf(reachDisplay)));
            comboCounter = Boolean.parseBoolean(props.getProperty("comboCounter", String.valueOf(comboCounter)));
            itemPhysics = Boolean.parseBoolean(props.getProperty("itemPhysics", String.valueOf(itemPhysics)));
            hideTitles = Boolean.parseBoolean(props.getProperty("hideTitles", String.valueOf(hideTitles)));
            reachDisplayX = Integer.parseInt(props.getProperty("reachDisplayX", String.valueOf(reachDisplayX)));
            reachDisplayY = Integer.parseInt(props.getProperty("reachDisplayY", String.valueOf(reachDisplayY)));
            comboDisplayX = Integer.parseInt(props.getProperty("comboDisplayX", String.valueOf(comboDisplayX)));
            comboDisplayY = Integer.parseInt(props.getProperty("comboDisplayY", String.valueOf(comboDisplayY)));

            // Booleanos Nuevos
            showHeldItem = Boolean.parseBoolean(props.getProperty("showHeldItem", String.valueOf(showHeldItem)));
            showServerHUD = Boolean.parseBoolean(props.getProperty("showServerHUD", String.valueOf(showServerHUD)));
            showCompass = Boolean.parseBoolean(props.getProperty("showCompass", String.valueOf(showCompass)));
            showNametagLogo = Boolean.parseBoolean(props.getProperty("showNametagLogo", String.valueOf(showNametagLogo)));
            showNametagLogoOthers = Boolean.parseBoolean(props.getProperty("showNametagLogoOthers", String.valueOf(showNametagLogoOthers)));
            showHardwareHud = Boolean.parseBoolean(props.getProperty("showHardwareHud", String.valueOf(showHardwareHud)));
            showMusicHud = Boolean.parseBoolean(props.getProperty("showMusicHud", String.valueOf(showMusicHud)));
            showMusicAlbumArt = Boolean.parseBoolean(props.getProperty("showMusicAlbumArt", String.valueOf(showMusicAlbumArt)));
            musicHudAlpha = Integer.parseInt(props.getProperty("musicHudAlpha", String.valueOf(musicHudAlpha)));
            showTntCountdown = Boolean.parseBoolean(props.getProperty("showTntCountdown", String.valueOf(showTntCountdown)));
            showBedwarsResources = Boolean.parseBoolean(props.getProperty("showBedwarsResources", String.valueOf(showBedwarsResources)));
            bwResTransparentBg = Boolean.parseBoolean(props.getProperty("bwResTransparentBg", String.valueOf(bwResTransparentBg)));
            forceItem3d = Boolean.parseBoolean(props.getProperty("forceItem3d", String.valueOf(forceItem3d)));
            overlayHudX = Integer.parseInt(props.getProperty("overlayHudX", String.valueOf(overlayHudX)));
            overlayHudY = Integer.parseInt(props.getProperty("overlayHudY", String.valueOf(overlayHudY)));
            overlayHudW = Integer.parseInt(props.getProperty("overlayHudW", String.valueOf(overlayHudW)));
            bwResX = Integer.parseInt(props.getProperty("bwResX", String.valueOf(bwResX)));
            bwResY = Integer.parseInt(props.getProperty("bwResY", String.valueOf(bwResY)));
            PerformanceConfig.loadFromProperties(props);
            if (PerformanceConfig.boostFps && PerformanceConfig.applyVanillaPreset) {
                OptifinePreset.applyIfEnabled();
            }

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