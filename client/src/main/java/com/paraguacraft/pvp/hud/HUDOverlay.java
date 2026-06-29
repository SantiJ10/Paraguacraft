package com.paraguacraft.pvp.hud;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.ScaledResolution;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.texture.DynamicTexture;
import net.minecraft.client.resources.I18n;
import net.minecraft.client.settings.KeyBinding;
import net.minecraft.enchantment.Enchantment;
import net.minecraft.enchantment.EnchantmentHelper;
import net.minecraft.item.ItemStack;
import net.minecraft.potion.Potion;
import net.minecraft.potion.PotionEffect;
import net.minecraft.util.ResourceLocation;
import net.minecraftforge.client.event.RenderGameOverlayEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import com.paraguacraft.pvp.hud.AdvancedHud;
import com.paraguacraft.pvp.hud.HudDraw;
import com.paraguacraft.pvp.gui.theme.TextUtil;
import com.paraguacraft.pvp.gui.theme.UiTheme;
import com.paraguacraft.pvp.modules.ModConfig;
import com.paraguacraft.pvp.modules.CombatStats;
import java.util.Collection;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import java.util.HashMap;
import java.awt.image.BufferedImage;
import java.io.ByteArrayInputStream;
import javax.imageio.ImageIO;
import org.apache.commons.codec.binary.Base64;
import org.lwjgl.input.Keyboard;
import org.lwjgl.input.Mouse;
import org.lwjgl.opengl.Display;
import org.lwjgl.opengl.GL11;
import com.paraguacraft.pvp.core.ModConfigApply;

public class HUDOverlay extends Gui {
    private final Minecraft mc = Minecraft.getMinecraft();
    private final int colorParagua = 0x00E5FF; // Cian
    private List<Long> leftClicks = new ArrayList<Long>();
    
    private static final ResourceLocation LOGO_GENERICO = new ResourceLocation("textures/items/ender_pearl.png");
    private static final ResourceLocation INVENTORY_TEXTURE = new ResourceLocation("textures/gui/container/inventory.png");
    private static final Map<String, ResourceLocation> serverIconCache = new HashMap<String, ResourceLocation>();

    @SubscribeEvent
    public void onRender(RenderGameOverlayEvent.Text event) {
        if (!ModConfig.loaded) {
            ModConfig.load();
            ModConfigApply.onStartup();
        }
        
        if (mc.gameSettings.showDebugInfo || mc.thePlayer == null) return;

        if (ModConfig.showFPS) {
            HudDraw.labeled("FPS: ", String.valueOf(Minecraft.getDebugFPS()), ModConfig.fpsX, ModConfig.fpsY);
        }
        if (ModConfig.showPing) {
            int ping = 0;
            if (mc.getNetHandler() != null && mc.getNetHandler().getPlayerInfo(mc.thePlayer.getUniqueID()) != null) {
                ping = mc.getNetHandler().getPlayerInfo(mc.thePlayer.getUniqueID()).getResponseTime();
            }
            HudDraw.labeled("Ping: ", (ping < 0 ? 0 : ping) + " ms", ModConfig.pingX, ModConfig.pingY);
        }
        if (ModConfig.showCPS) {
            calculateCPS();
            HudDraw.labeled("CPS: ", String.valueOf(leftClicks.size()), ModConfig.cpsX, ModConfig.cpsY);
        }
        if (ModConfig.showCoords) {
            String coords = String.format("X: %.0f  Y: %.0f  Z: %.0f", mc.thePlayer.posX, mc.thePlayer.posY, mc.thePlayer.posZ);
            HudDraw.text(coords, ModConfig.coordsX, ModConfig.coordsY, UiTheme.ACCENT);
        }
        
        if (ModConfig.showKeystrokes) drawKeystrokes();
        if (ModConfig.showArmor) drawArmorStatus();
        
        // --- Módulos Premium (Lunar Style 100% Transparentes) ---
        if (ModConfig.showPotions) drawPotionStatus(); 
        if (ModConfig.showHeldItem) drawHeldItemMod(); 
        if (ModConfig.showServerHUD) drawServerHUD(); 
        if (ModConfig.showCompass) drawCompass();

        AdvancedHud.drawOverlay();
        AdvancedHud.drawBedwarsResources(mc.thePlayer);
        drawCombatStats();
    }

    private void drawCombatStats() {
        if (ModConfig.reachDisplay && CombatStats.lastReach > 0.0) {
            HudDraw.labeled("Reach: ", String.format("%.2f", CombatStats.lastReach), ModConfig.reachDisplayX, ModConfig.reachDisplayY);
        }
        if (ModConfig.comboCounter && CombatStats.comboCount > 0) {
            HudDraw.labeled("Combo: ", String.valueOf(CombatStats.comboCount), ModConfig.comboDisplayX, ModConfig.comboDisplayY);
        }
    }

    // ============================================================
    // --- COMPASS HUD (Brújula Detallada SIN FONDO) ---
    // ============================================================
    private void drawCompass() {
        ScaledResolution sr = new ScaledResolution(mc);
        int centerX = sr.getScaledWidth() / 2;
        int y = ModConfig.compassY;
        int boxW = 220; 
        int boxH = 16;
        int x = centerX - (boxW / 2);
        
        // SCISSOR TEST (Recorta los números que salen de los bordes)
        GL11.glEnable(GL11.GL_SCISSOR_TEST);
        int factor = sr.getScaleFactor();
        GL11.glScissor(x * factor, Display.getHeight() - (y + boxH) * factor, boxW * factor, boxH * factor);

        float yaw = mc.thePlayer.rotationYaw % 360.0F;
        if (yaw < 0) yaw += 360.0F;

        // Renderizamos marcas en un rango de visión de 60 grados
        for (int i = (int)yaw - 60; i < (int)yaw + 60; i++) {
            int angle = (i + 360) % 360;
            
            // Marcas numéricas y cardinales cada 15 grados
            if (angle % 15 == 0) {
                float offset = i - yaw;
                float px = centerX + (offset * 2.0F); // Factor de separación horizontal

                String markerText = "";
                boolean isCardinal = false;
                boolean isIntercardinal = false;

                if (angle == 0 || angle == 360) { markerText = "S"; isCardinal = true; }
                else if (angle == 45) { markerText = "SW"; isIntercardinal = true; }
                else if (angle == 90) { markerText = "W"; isCardinal = true; }
                else if (angle == 135) { markerText = "NW"; isIntercardinal = true; }
                else if (angle == 180) { markerText = "N"; isCardinal = true; }
                else if (angle == 225) { markerText = "NE"; isIntercardinal = true; }
                else if (angle == 270) { markerText = "E"; isCardinal = true; }
                else if (angle == 315) { markerText = "SE"; isIntercardinal = true; }
                else { markerText = String.valueOf(angle); } // Grados como 105, 120, 150...

                int color;
                if (isCardinal) color = 0xFF00E5FF; // Cian
                else if (isIntercardinal) color = 0xFFFFFFFF; // Blanco
                else color = 0xFFAAAAAA; // Gris claro para números

                int textW = HudDraw.width(markerText);
                HudDraw.text(markerText, px - textW / 2f, y + 6, color);
            }
        }
        
        GL11.glDisable(GL11.GL_SCISSOR_TEST);
        
        // Marcador central (Triángulo apuntando abajo)
        Gui.drawRect(centerX - 1, y, centerX + 1, y + 4, colorParagua);
    }

    // ============================================================
    // --- POCIONES LUNAR STYLE (ÍCONO REAL, SIN FONDO GRIS) ---
    // ============================================================
    private void drawPotionStatus() {
        Collection<PotionEffect> effects = mc.thePlayer.getActivePotionEffects();
        if (effects.isEmpty()) return;
        
        int yOffset = 0;
        int x = ModConfig.potionX;
        
        for (PotionEffect effect : effects) {
            Potion potion = Potion.potionTypes[effect.getPotionID()];
            int y = ModConfig.potionY + yOffset;

            // 1. Dibujar el Ícono Real de la Poción
            GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
            mc.getTextureManager().bindTexture(INVENTORY_TEXTURE);
            if (potion.hasStatusIcon()) {
                int iconIndex = potion.getStatusIconIndex();
                int u = iconIndex % 8 * 18;
                int v = 198 + iconIndex / 8 * 18;
                Gui.drawModalRectWithCustomSizedTexture(x, y, u, v, 18, 18, 256, 256);
            }

            // 2. Textos (Blanco para nombre, Gris para tiempo)
            String name = I18n.format(potion.getName());
            if (effect.getAmplifier() > 0) name += " " + (effect.getAmplifier() + 1);
            String duration = Potion.getDurationString(effect);

            HudDraw.text(name, x + 22, y + 1, UiTheme.TEXT);
            HudDraw.text(duration, x + 22, y + 11, UiTheme.TEXT_DIM);
            
            yOffset += 24; // Espacio para la siguiente poción
        }
    }

    // ============================================================
    // --- HELD ITEM MOD (SIN FONDO, LETRAS GRISES) ---
    // ============================================================
    private void drawHeldItemMod() {
        ItemStack held = mc.thePlayer.getHeldItem();
        if (held == null) return;

        int x = ModConfig.heldX;
        int y = ModConfig.heldY;

        // Ícono del Arma 3D
        GlStateManager.pushMatrix();
        GlStateManager.enableDepth();
        mc.getRenderItem().renderItemAndEffectIntoGUI(held, x, y);
        GlStateManager.disableDepth();
        GlStateManager.disableLighting();
        GlStateManager.popMatrix();

        int textX = x + 20;
        int textY = y + 4;

        String displayName = TextUtil.stripColor(held.getDisplayName());
        HudDraw.text(displayName, textX, textY, held.isItemEnchanted() ? UiTheme.ACCENT : UiTheme.TEXT);
        textY += 10;

        Map<Integer, Integer> enchants = EnchantmentHelper.getEnchantments(held);
        if (!enchants.isEmpty()) {
            for (Map.Entry<Integer, Integer> entry : enchants.entrySet()) {
                Enchantment ench = Enchantment.getEnchantmentById(entry.getKey());
                if (ench != null) {
                    String enchName = ench.getTranslatedName(entry.getValue());
                    String roman = enchName.replaceAll(" 1$", " I").replaceAll(" 2$", " II").replaceAll(" 3$", " III").replaceAll(" 4$", " IV").replaceAll(" 5$", " V");
                    HudDraw.text(TextUtil.stripColor(roman), textX, textY, UiTheme.TEXT_DIM);
                    textY += 10;
                }
            }
        }
    }

    // ============================================================
    // --- SERVER HUD (ÍCONO REAL, CACHÉ ARREGLADA Y BLEND ACTIVADO) ---
    // ============================================================
    private void drawServerHUD() {
        if (mc.isIntegratedServerRunning() || mc.getCurrentServerData() == null) return; 

        int x = ModConfig.serverX;
        int y = ModConfig.serverY;
        String ip = mc.getCurrentServerData().serverIP;
        
        ResourceLocation serverIcon = LOGO_GENERICO; 
        String base64Icon = mc.getCurrentServerData().getBase64EncodedIconData();

        // 1. Buscamos agresivamente el Favicon en el disco (servers.dat)
        if (base64Icon == null || base64Icon.isEmpty()) {
            try {
                net.minecraft.client.multiplayer.ServerList list = new net.minecraft.client.multiplayer.ServerList(mc);
                list.loadServerList();
                for (int i = 0; i < list.countServers(); ++i) {
                    net.minecraft.client.multiplayer.ServerData d = list.getServerData(i);
                    // Comprobación flexible por si entras por un proxy o IP secundaria
                    if (d.serverIP.toLowerCase().contains(ip.toLowerCase()) && d.getBase64EncodedIconData() != null) {
                        base64Icon = d.getBase64EncodedIconData();
                        break;
                    }
                }
            } catch (Exception e) {} 
        }

        // 2. Decodificación de Imagen Base64
        if (base64Icon != null && !base64Icon.isEmpty()) {
            if (serverIconCache.containsKey(ip)) {
                serverIcon = serverIconCache.get(ip);
            } else {
                try {
                    String decodedData = base64Icon;
                    // Limpiamos la cabecera por las dudas
                    if (decodedData.startsWith("data:image/png;base64,")) {
                        decodedData = decodedData.substring("data:image/png;base64,".length());
                    }
                    
                    byte[] imageBytes = org.apache.commons.codec.binary.Base64.decodeBase64(decodedData);
                    BufferedImage image = ImageIO.read(new ByteArrayInputStream(imageBytes));
                    
                    if (image != null) {
                        DynamicTexture texture = new DynamicTexture(image);
                        ResourceLocation loc = mc.getTextureManager().getDynamicTextureLocation("server_icon_" + ip.replace(".", "_"), texture);
                        serverIconCache.put(ip, loc); 
                        serverIcon = loc;
                    }
                } catch (Exception e) {
                    // MAGIA: No guardamos la Ender Pearl en caché si falla. 
                    // Lo dejamos que lo vuelva a intentar en el próximo frame.
                }
            }
        }

        // 3. Renderizado del Ícono
        GlStateManager.pushMatrix();
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);

        if (serverIcon == LOGO_GENERICO) {
            // Si es la Ender Pearl, la dibujamos como un Ítem del inventario
            GlStateManager.enableDepth();
            mc.getRenderItem().renderItemIntoGUI(new ItemStack(net.minecraft.init.Items.ender_pearl), x, y);
            GlStateManager.disableDepth();
            GlStateManager.disableLighting(); // Apagamos la luz para que no oscurezca el texto
        } else {
            // Si es el logo real, lo dibujamos como una textura 2D con transparencia (Blend)
            GlStateManager.enableBlend();
            GlStateManager.enableAlpha();
            GlStateManager.blendFunc(GL11.GL_SRC_ALPHA, GL11.GL_ONE_MINUS_SRC_ALPHA);
            
            mc.getTextureManager().bindTexture(serverIcon);
            Gui.drawScaledCustomSizeModalRect(x, y, 0, 0, 64, 64, 16, 16, 64, 64);
            
            GlStateManager.disableBlend();
        }
        GlStateManager.popMatrix();

        // 4. IP Limpia sin rectángulos
        HudDraw.text(ip, x + 20, y + 4, UiTheme.TEXT);
    }

    // ============================================================
    // --- MÉTODOS ORIGINALES (NO TOCAR) ---
    // ============================================================
    private void drawKeystrokes() {
        int x = ModConfig.keysX;
        int y = ModConfig.keysY;
        int gap = 2, size = 20;
        drawKey(x + size + gap, y, size, size, mc.gameSettings.keyBindForward);
        drawKey(x, y + size + gap, size, size, mc.gameSettings.keyBindLeft); 
        drawKey(x + size + gap, y + size + gap, size, size, mc.gameSettings.keyBindBack); 
        drawKey(x + (size + gap) * 2, y + size + gap, size, size, mc.gameSettings.keyBindRight); 
        drawMouseKey(x, y + (size + gap) * 2, 31, size, 0, "LMB"); 
        drawMouseKey(x + 31 + gap, y + (size + gap) * 2, 31, size, 1, "RMB"); 
    }

    private void drawKey(int x, int y, int w, int h, KeyBinding key) {
        boolean pressed = Keyboard.isKeyDown(key.getKeyCode());
        int bg = pressed ? 0x88FFFFFF : 0x88000000;
        int fg = pressed ? 0x000000 : 0xFFFFFF;
        Gui.drawRect(x, y, x + w, y + h, bg);
        String name = Keyboard.getKeyName(key.getKeyCode());
        HudDraw.centered(name, x + w / 2f, y + h / 2f - 4, fg);
    }

    private void drawMouseKey(int x, int y, int w, int h, int button, String name) {
        boolean pressed = Mouse.isButtonDown(button);
        int bg = pressed ? 0x88FFFFFF : 0x88000000;
        int fg = pressed ? 0x000000 : 0xFFFFFF;
        Gui.drawRect(x, y, x + w, y + h, bg);
        HudDraw.centered(name, x + w / 2f, y + h / 2f - 4, fg);
    }

    private void drawArmorStatus() {
        int yOffset = 0;
        for (int i = 3; i >= 0; i--) {
            ItemStack stack = mc.thePlayer.inventory.armorInventory[i];
            if (stack != null) {
                int y = ModConfig.armorY + yOffset;
                mc.getRenderItem().renderItemAndEffectIntoGUI(stack, ModConfig.armorX + 25, y);
                GlStateManager.disableLighting();
                GlStateManager.disableDepth();
                if (ModConfig.showArmorPercentage && stack.isItemStackDamageable()) {
                    int maxDam = stack.getMaxDamage();
                    int currDam = stack.getItemDamage();
                    int percent = (maxDam > 0) ? (int) (((maxDam - currDam) * 100.0F) / maxDam) : 100;
                    String text = percent + "%";
                    int color = percent < 25 ? 0xFFFF5555 : (percent < 50 ? 0xFFFFCC55 : 0xFF55FF55);
                    HudDraw.text(text, ModConfig.armorX + 22 - HudDraw.width(text), y + 4, color);
                }
                GlStateManager.enableDepth();
                GlStateManager.enableLighting();
                yOffset += 16;
            }
        }
        GlStateManager.disableLighting();
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
    }

    private void calculateCPS() {
        long time = System.currentTimeMillis();
        if (Mouse.isButtonDown(0)) {
            if (leftClicks.isEmpty() || time - leftClicks.get(leftClicks.size() - 1) > 50) {
                leftClicks.add(time);
            }
        }
        leftClicks.removeIf(click -> time - click > 1000);
    }
}