package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.gui.theme.TextUtil;
import com.paraguacraft.pvp.gui.theme.UiEasing;
import com.paraguacraft.pvp.gui.theme.UiTheme;
import com.paraguacraft.pvp.core.OptifinePreset;
import com.paraguacraft.pvp.core.PerformanceConfig;
import com.paraguacraft.pvp.core.BorderlessWindowManager;
import com.paraguacraft.pvp.core.ModConfigFeedback;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.renderer.GlStateManager;
import org.lwjgl.input.Keyboard;

import java.io.IOException;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Mod Menu estilo Lunar — panel translúcido, categorías, buscador y tarjetas con toggle.
 */
public class GuiParaguaMenu extends GuiScreen {

    private static final int SIDEBAR = 148;
    private static final int TOPBAR = 52;
    private static final int CARD_W = 168;
    private static final int CARD_H = 72;
    private static final int GAP = 12;

    private static final String[] CATEGORY_IDS = {"all", "hud", "pvp", "mechanics", "server", "textures", "perf"};
    private static final String[] CATEGORY_LABELS = {"Todos", "HUD", "PvP", "Mecanicas", "Servidor", "Texturas", "Rendimiento"};

    private int selectedCategory;
    private String searchQuery = "";
    private boolean searchFocused = true;
    private final Map<Integer, Float> toggleAnim = new HashMap<Integer, Float>();

    private static final class ModEntry {
        final int id;
        final String name;
        final int category;
        final String icon;

        ModEntry(int id, String name, int category) {
            this.id = id;
            this.name = name;
            this.category = category;
            this.icon = name.isEmpty() ? "?" : name.substring(0, 1).toUpperCase();
        }
    }

    private static final ModEntry[] ALL_MODS = {
        new ModEntry(0, "FPS", 1),
        new ModEntry(1, "Ping", 1),
        new ModEntry(2, "CPS", 1),
        new ModEntry(3, "Keystrokes", 1),
        new ModEntry(5, "Coordenadas", 1),
        new ModEntry(6, "Armor HUD", 1),
        new ModEntry(7, "% Armadura", 1),
        new ModEntry(8, "Potion HUD", 1),
        new ModEntry(12, "Held Item", 1),
        new ModEntry(14, "Server HUD", 4),
        new ModEntry(15, "Brújula", 1),
        new ModEntry(16, "Resource Packs", 5),
        new ModEntry(17, "Nametag Logo", 4),
        new ModEntry(18, "Logo en otros", 4),
        new ModEntry(4, "No Hurt Cam", 2),
        new ModEntry(9, "Scoreboard", 2),
        new ModEntry(10, "Toggle Sneak", 3),
        new ModEntry(11, "Dynamic FOV", 3),
        new ModEntry(13, "Borderless", 3),
        new ModEntry(20, "Old Animations", 3),
        new ModEntry(19, "Boost FPS", 6),
        new ModEntry(21, "Entity Cull", 6),
        new ModEntry(22, "Nametag Cull", 6),
        new ModEntry(23, "Particulas", 6),
        new ModEntry(24, "Block Entity", 6),
        new ModEntry(25, "Anim Cull", 6),
        new ModEntry(26, "Memory Clean", 6),
        new ModEntry(27, "OptiFine Preset", 6),
        new ModEntry(28, "Armor Stand", 6),
        new ModEntry(29, "Item Frame", 6),
        new ModEntry(30, "Nametag LOD", 6),
        new ModEntry(31, "Skip Combat FX", 6),
        new ModEntry(32, "HW Preset", 6),
        new ModEntry(33, "Perfiles", 3),
        new ModEntry(34, "Keybinds", 3),
    };

    @Override
    public void initGui() {
        searchFocused = true;
    }

    private FontRenderer fr() {
        return this.fontRendererObj;
    }

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        FontRenderer fr = fr();
        drawRect(0, 0, width, height, 0x99000000);

        int panelX = Math.max(8, width / 2 - 420);
        int panelY = 24;
        int panelW = Math.min(width - 16, 840);
        int panelH = height - 48;
        Gui.drawRect(panelX, panelY, panelX + panelW, panelY + panelH, 0xCC0A0C14);
        Gui.drawRect(panelX, panelY, panelX + panelW, panelY + 1, 0x33FFFFFF);
        Gui.drawRect(panelX, panelY + panelH - 1, panelX + panelW, panelY + panelH, 0x22FFFFFF);

        drawSidebar(panelX, panelY, panelH, mouseX, mouseY);
        drawTopbar(panelX + SIDEBAR, panelY, panelW - SIDEBAR, mouseX, mouseY);
        drawModGrid(panelX + SIDEBAR, panelY + TOPBAR, panelW - SIDEBAR - 12, panelH - TOPBAR - 12, mouseX, mouseY);

        fr.drawStringWithShadow(
            "Right Shift - Mod Menu  |  Right Ctrl - Editar HUD  |  Texturas - Resource Packs",
            panelX + 12,
            panelY + panelH - 14,
            UiTheme.TEXT_DIM
        );
        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    private void drawSidebar(int x, int y, int h, int mouseX, int mouseY) {
        FontRenderer fr = fr();
        Gui.drawRect(x, y, x + SIDEBAR, y + h, 0xDD080A10);
        fr.drawStringWithShadow("PARAGUACRAFT", x + 14, y + 16, UiTheme.ACCENT);
        fr.drawStringWithShadow("Client V2 - Mods", x + 14, y + 38, UiTheme.TEXT_DIM);

        int catY = y + 64;
        for (int i = 0; i < CATEGORY_IDS.length; i++) {
            boolean selected = i == selectedCategory;
            boolean hover = mouseX >= x && mouseX <= x + SIDEBAR && mouseY >= catY && mouseY <= catY + 28;
            if (selected) {
                Gui.drawRect(x, catY, x + SIDEBAR, catY + 28, 0x4400E5FF);
                Gui.drawRect(x, catY, x + 3, catY + 28, UiTheme.ACCENT);
            } else if (hover) {
                Gui.drawRect(x, catY, x + SIDEBAR, catY + 28, 0x18FFFFFF);
            }
            fr.drawStringWithShadow(
                CATEGORY_LABELS[i],
                x + 18,
                catY + 9,
                selected ? UiTheme.TEXT : UiTheme.TEXT_DIM
            );
            catY += 28;
        }
    }

    private void drawTopbar(int x, int y, int w, int mouseX, int mouseY) {
        FontRenderer fr = fr();
        Gui.drawRect(x, y, x + w, y + TOPBAR, 0x88101018);
        int searchX = x + 16;
        int searchY = y + 14;
        int searchW = Math.min(280, w - 32);
        boolean hoverSearch = mouseX >= searchX && mouseX <= searchX + searchW && mouseY >= searchY && mouseY <= searchY + 24;
        Gui.drawRect(searchX, searchY, searchX + searchW, searchY + 24, hoverSearch || searchFocused ? 0xAA121820 : 0x88000000);
        if (searchFocused) {
            Gui.drawRect(searchX, searchY + 23, searchX + searchW, searchY + 24, UiTheme.ACCENT);
        }
        String shown = searchQuery.isEmpty() ? "Buscar mods..." : searchQuery;
        int color = searchQuery.isEmpty() ? UiTheme.TEXT_DIM : UiTheme.TEXT;
        fr.drawStringWithShadow(shown + (searchFocused && (System.currentTimeMillis() / 500) % 2 == 0 ? "_" : ""), searchX + 8, searchY + 7, color);
    }

    private void drawModGrid(int x, int y, int w, int h, int mouseX, int mouseY) {
        List<ModEntry> visible = filteredMods();
        int col = Math.max(1, (w + GAP) / (CARD_W + GAP));
        int cx = 0;
        int cy = 0;
        for (ModEntry mod : visible) {
            int cardX = x + 8 + cx * (CARD_W + GAP);
            int cardY = y + 8 + cy * (CARD_H + GAP);
            if (cardY + CARD_H > y + h) {
                break;
            }
            drawModCard(mod, cardX, cardY, getModState(mod.id), mouseX, mouseY);
            cx++;
            if (cx >= col) {
                cx = 0;
                cy++;
            }
        }
        if (visible.isEmpty()) {
            FontRenderer fr = fr();
            fr.drawStringWithShadow("Sin mods en esta categoria", x + w / 2 - fr.getStringWidth("Sin mods en esta categoria") / 2, y + h / 2, UiTheme.TEXT_DIM);
        }
    }

    private void drawModCard(ModEntry mod, int x, int y, boolean enabled, int mouseX, int mouseY) {
        FontRenderer fr = fr();
        boolean isScreen = mod.id == 16 || mod.id == 33 || mod.id == 34;
        boolean hover = mouseX >= x && mouseX <= x + CARD_W && mouseY >= y && mouseY <= y + CARD_H;
        float target = isScreen ? 1f : (enabled ? 1f : 0f);
        float anim = toggleAnim.containsKey(mod.id) ? toggleAnim.get(mod.id) : target;
        anim = UiEasing.approach(anim, target, 0.22f);
        toggleAnim.put(mod.id, anim);

        int bg = hover ? 0xCC161A24 : 0xAA101218;
        Gui.drawRect(x, y, x + CARD_W, y + CARD_H, bg);
        Gui.drawRect(x, y, x + CARD_W, y + 1, 0x28FFFFFF);

        Gui.drawRect(x + 10, y + 10, x + 42, y + 42, 0x3300E5FF);
        fr.drawStringWithShadow(mod.icon, x + 22 - fr.getStringWidth(mod.icon) / 2, y + 18, UiTheme.ACCENT);
        fr.drawStringWithShadow(modDisplayName(mod.id), x + 50, y + 16, isScreen || enabled ? UiTheme.TEXT : UiTheme.TEXT_DIM);

        int toggleY = y + CARD_H - 22;
        if (isScreen) {
            Gui.drawRect(x + 8, toggleY, x + CARD_W - 8, toggleY + 16, hover ? UiTheme.ACCENT : 0xFF226688);
            fr.drawStringWithShadow("ABRIR", x + CARD_W / 2 - fr.getStringWidth("ABRIR") / 2, toggleY + 4, 0xFFFFFF);
        } else {
            int toggleColor = lerpColor(0xFFCC4444, 0xFF22CC66, UiEasing.easeOutCubic(anim));
            Gui.drawRect(x + 8, toggleY, x + CARD_W - 8, toggleY + 16, toggleColor);
            String stateLabel = modStateLabel(mod.id, enabled);
            fr.drawStringWithShadow(stateLabel, x + CARD_W / 2 - fr.getStringWidth(stateLabel) / 2, toggleY + 4, 0xFFFFFF);
        }
    }

    private String modDisplayName(int id) {
        if (id == 23) {
            return "Particulas: " + PerformanceConfig.particleMode.getLabel();
        }
        if (id == 32) {
            return "HW: " + com.paraguacraft.pvp.core.HardwarePreset.getDetectedTier().name();
        }
        for (ModEntry mod : ALL_MODS) {
            if (mod.id == id) {
                return mod.name;
            }
        }
        return "?";
    }

    private String modStateLabel(int id, boolean enabled) {
        if (id == 23) {
            return PerformanceConfig.particleMode.getLabel().toUpperCase();
        }
        return enabled ? "ON" : "OFF";
    }

    private List<ModEntry> filteredMods() {
        List<ModEntry> out = new ArrayList<ModEntry>();
        for (ModEntry mod : ALL_MODS) {
            if (selectedCategory > 0 && mod.category != selectedCategory) {
                continue;
            }
            if (!searchQuery.isEmpty()) {
                String label = modDisplayName(mod.id);
                if (!TextUtil.containsIgnoreCase(mod.name, searchQuery)
                    && !TextUtil.containsIgnoreCase(label, searchQuery)) {
                    continue;
                }
            }
            out.add(mod);
        }
        return out;
    }

    @Override
    protected void mouseClicked(int mouseX, int mouseY, int mouseButton) throws IOException {
        if (mouseButton != 0) {
            return;
        }
        int panelX = Math.max(8, width / 2 - 420);
        int panelY = 24;
        int panelW = Math.min(width - 16, 840);
        int panelH = height - 48;

        int searchX = panelX + SIDEBAR + 16;
        int searchY = panelY + 14;
        int searchW = Math.min(280, panelW - SIDEBAR - 32);
        if (mouseX >= searchX && mouseX <= searchX + searchW && mouseY >= searchY && mouseY <= searchY + 24) {
            searchFocused = true;
            return;
        }
        searchFocused = false;

        int catY = panelY + 64;
        for (int i = 0; i < CATEGORY_IDS.length; i++) {
            if (mouseX >= panelX && mouseX <= panelX + SIDEBAR && mouseY >= catY && mouseY <= catY + 28) {
                selectedCategory = i;
                return;
            }
            catY += 28;
        }

        List<ModEntry> visible = filteredMods();
        int gridX = panelX + SIDEBAR;
        int gridY = panelY + TOPBAR;
        int gridW = panelW - SIDEBAR - 12;
        int col = Math.max(1, (gridW + GAP) / (CARD_W + GAP));
        int cx = 0;
        int cy = 0;
        for (ModEntry mod : visible) {
            int cardX = gridX + 8 + cx * (CARD_W + GAP);
            int cardY = gridY + 8 + cy * (CARD_H + GAP);
            if (mouseX >= cardX && mouseX <= cardX + CARD_W && mouseY >= cardY && mouseY <= cardY + CARD_H) {
                if (mod.id == 16) {
                    mc.displayGuiScreen(new GuiResourcePacks());
                } else if (mod.id == 33) {
                    mc.displayGuiScreen(new GuiModProfiles());
                } else if (mod.id == 34) {
                    mc.displayGuiScreen(new GuiKeybinds());
                } else {
                    toggleMod(mod.id);
                    ModConfig.save();
                }
                return;
            }
            cx++;
            if (cx >= col) {
                cx = 0;
                cy++;
            }
        }
    }

    @Override
    protected void keyTyped(char typedChar, int keyCode) throws IOException {
        if (keyCode == Keyboard.KEY_ESCAPE) {
            mc.displayGuiScreen(null);
            return;
        }
        if (!searchFocused) {
            super.keyTyped(typedChar, keyCode);
            return;
        }
        if (keyCode == Keyboard.KEY_BACK) {
            if (!searchQuery.isEmpty()) {
                searchQuery = searchQuery.substring(0, searchQuery.length() - 1);
            }
            return;
        }
        if (Character.isISOControl(typedChar)) {
            return;
        }
        if (searchQuery.length() < 32) {
            searchQuery += typedChar;
        }
    }

    private static int lerpColor(int a, int b, float t) {
        int aa = (a >> 24) & 0xFF, ar = (a >> 16) & 0xFF, ag = (a >> 8) & 0xFF, ab = a & 0xFF;
        int ba = (b >> 24) & 0xFF, br = (b >> 16) & 0xFF, bg = (b >> 8) & 0xFF, bb = b & 0xFF;
        return ((int) (aa + (ba - aa) * t) << 24)
            | ((int) (ar + (br - ar) * t) << 16)
            | ((int) (ag + (bg - ag) * t) << 8)
            | (int) (ab + (bb - ab) * t);
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
            case 16: return true;
            case 17: return ModConfig.showNametagLogo;
            case 18: return ModConfig.showNametagLogoOthers;
            case 19: return PerformanceConfig.boostFps;
            case 20: return PerformanceConfig.oldAnimations;
            case 21: return PerformanceConfig.entityCull;
            case 22: return PerformanceConfig.nametagCull;
            case 23: return PerformanceConfig.particleMode != PerformanceConfig.ParticleMode.OFF;
            case 24: return PerformanceConfig.blockEntityCull;
            case 25: return PerformanceConfig.entityAnimCull;
            case 26: return PerformanceConfig.memoryCleanupOnWorldChange;
            case 27: return PerformanceConfig.applyVanillaPreset;
            case 28: return PerformanceConfig.armorStandCull;
            case 29: return PerformanceConfig.itemFrameCull;
            case 30: return PerformanceConfig.nametagLod;
            case 31: return PerformanceConfig.skipCombatFx;
            case 32: return PerformanceConfig.hardwareAutoPreset;
            case 33: return true;
            case 34: return true;
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
            case 17: ModConfig.showNametagLogo = !ModConfig.showNametagLogo; break;
            case 18: ModConfig.showNametagLogoOthers = !ModConfig.showNametagLogoOthers; break;
            case 19:
                PerformanceConfig.setBoostFps(!PerformanceConfig.boostFps);
                ModConfig.save();
                break;
            case 20:
                PerformanceConfig.oldAnimations = !PerformanceConfig.oldAnimations;
                ModConfig.save();
                break;
            case 21:
                PerformanceConfig.entityCull = !PerformanceConfig.entityCull;
                ModConfig.save();
                break;
            case 22:
                PerformanceConfig.nametagCull = !PerformanceConfig.nametagCull;
                ModConfig.save();
                break;
            case 23:
                PerformanceConfig.cycleParticleMode();
                ModConfig.save();
                break;
            case 24:
                PerformanceConfig.blockEntityCull = !PerformanceConfig.blockEntityCull;
                ModConfig.save();
                break;
            case 25:
                PerformanceConfig.entityAnimCull = !PerformanceConfig.entityAnimCull;
                ModConfig.save();
                break;
            case 26:
                PerformanceConfig.memoryCleanupOnWorldChange = !PerformanceConfig.memoryCleanupOnWorldChange;
                ModConfig.save();
                break;
            case 27:
                PerformanceConfig.applyVanillaPreset = !PerformanceConfig.applyVanillaPreset;
                if (PerformanceConfig.applyVanillaPreset) {
                    OptifinePreset.applyIfEnabled();
                }
                ModConfig.save();
                break;
            case 13:
                ModConfig.borderlessWindow = !ModConfig.borderlessWindow;
                BorderlessWindowManager.apply(ModConfig.borderlessWindow);
                ModConfigFeedback.notifyToggle("Borderless", ModConfig.borderlessWindow);
                break;
            case 28:
                PerformanceConfig.armorStandCull = !PerformanceConfig.armorStandCull;
                ModConfig.save();
                break;
            case 29:
                PerformanceConfig.itemFrameCull = !PerformanceConfig.itemFrameCull;
                ModConfig.save();
                break;
            case 30:
                PerformanceConfig.nametagLod = !PerformanceConfig.nametagLod;
                ModConfig.save();
                break;
            case 31:
                PerformanceConfig.skipCombatFx = !PerformanceConfig.skipCombatFx;
                ModConfig.save();
                break;
            case 32:
                PerformanceConfig.hardwareAutoPreset = !PerformanceConfig.hardwareAutoPreset;
                if (PerformanceConfig.hardwareAutoPreset) {
                    com.paraguacraft.pvp.core.HardwarePreset.applyIfEnabled();
                }
                ModConfig.save();
                break;
            default: break;
        }
    }

    @Override
    public boolean doesGuiPauseGame() {
        return false;
    }
}
