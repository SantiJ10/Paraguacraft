package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.gui.theme.TextUtil;
import com.paraguacraft.pvp.gui.theme.UiEasing;
import com.paraguacraft.pvp.gui.theme.UiTheme;
import com.paraguacraft.pvp.core.OptifinePreset;
import com.paraguacraft.pvp.core.PerformanceConfig;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.gui.ScaledResolution;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.util.ResourceLocation;
import com.paraguacraft.pvp.core.ModLang;
import org.lwjgl.input.Keyboard;
import org.lwjgl.input.Mouse;
import org.lwjgl.opengl.GL11;

import java.io.IOException;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Mod Menu estilo Lunar — panel translúcido, categorías, buscador y tarjetas con toggle.
 */
public class GuiParaguaMenu extends GuiScreen {

    private static final ResourceLocation MOD_ICON = new ResourceLocation("paraguacraft", "textures/gui/logo.png");

    private static final int SIDEBAR = 148;
    private static final int TOPBAR = 52;
    private static final int CARD_W = 168;
    private static final int CARD_H = 72;
    private static final int GAP = 12;

    private static final String[] CATEGORY_IDS = {"all", "hud", "pvp", "mechanics", "server", "textures", "perf", "hypixel"};

    private int selectedCategory;
    private String searchQuery = "";
    private boolean searchFocused = true;
    private float scrollOffset = 0f;
    private int lastVisibleHash = 0;
    private final Map<Integer, Float> toggleAnim = new HashMap<Integer, Float>();

    private static final class ModEntry {
        final int id;
        final String langKey;
        final int category;

        ModEntry(int id, String langKey, int category) {
            this.id = id;
            this.langKey = langKey;
            this.category = category;
        }
    }

    private static final ModEntry[] ALL_MODS = {
        new ModEntry(0, "paraguacraft.menu.mod.fps", 1),
        new ModEntry(1, "paraguacraft.menu.mod.ping", 1),
        new ModEntry(2, "paraguacraft.menu.mod.cps", 1),
        new ModEntry(3, "paraguacraft.menu.mod.keystrokes", 1),
        new ModEntry(5, "paraguacraft.menu.mod.coords", 1),
        new ModEntry(6, "paraguacraft.menu.mod.armor", 1),
        new ModEntry(7, "paraguacraft.menu.mod.armor_pct", 1),
        new ModEntry(8, "paraguacraft.menu.mod.potions", 1),
        new ModEntry(12, "paraguacraft.menu.mod.held_item", 1),
        new ModEntry(14, "paraguacraft.menu.mod.server_hud", 4),
        new ModEntry(15, "paraguacraft.menu.mod.compass", 1),
        new ModEntry(16, "paraguacraft.menu.mod.resource_packs", 5),
        new ModEntry(17, "paraguacraft.menu.mod.nametag_logo", 4),
        new ModEntry(18, "paraguacraft.menu.mod.nametag_others", 4),
        new ModEntry(4, "paraguacraft.menu.mod.no_hurt_cam", 2),
        new ModEntry(9, "paraguacraft.menu.mod.scoreboard", 2),
        new ModEntry(10, "paraguacraft.menu.mod.toggle_sneak", 3),
        new ModEntry(53, "paraguacraft.menu.mod.toggle_sprint", 3),
        new ModEntry(54, "paraguacraft.menu.mod.toggle_sprint_legacy", 3),
        new ModEntry(11, "paraguacraft.menu.mod.dynamic_fov", 3),
        new ModEntry(13, "paraguacraft.menu.mod.windowed_fullscreen", 3),
        new ModEntry(20, "paraguacraft.menu.mod.old_anim", 3),
        new ModEntry(19, "paraguacraft.menu.mod.boost_fps", 6),
        new ModEntry(21, "paraguacraft.menu.mod.entity_cull", 6),
        new ModEntry(22, "paraguacraft.menu.mod.nametag_cull", 6),
        new ModEntry(23, "paraguacraft.menu.mod.particles", 6),
        new ModEntry(24, "paraguacraft.menu.mod.block_entity", 6),
        new ModEntry(25, "paraguacraft.menu.mod.anim_cull", 6),
        new ModEntry(26, "paraguacraft.menu.mod.memory_clean", 6),
        new ModEntry(27, "paraguacraft.menu.mod.optifine_preset", 6),
        new ModEntry(28, "paraguacraft.menu.mod.armor_stand", 6),
        new ModEntry(29, "paraguacraft.menu.mod.item_frame", 6),
        new ModEntry(30, "paraguacraft.menu.mod.nametag_lod", 6),
        new ModEntry(31, "paraguacraft.menu.mod.skip_combat_fx", 6),
        new ModEntry(32, "paraguacraft.menu.mod.hw_preset", 6),
        new ModEntry(33, "paraguacraft.menu.mod.profiles", 3),
        new ModEntry(34, "paraguacraft.menu.mod.keybinds", 3),
        new ModEntry(35, "paraguacraft.menu.mod.hardware_hud", 1),
        new ModEntry(36, "paraguacraft.menu.mod.music_hud", 1),
        new ModEntry(37, "paraguacraft.menu.mod.tnt_countdown", 2),
        new ModEntry(38, "paraguacraft.menu.mod.bw_resources", 2),
        new ModEntry(39, "paraguacraft.menu.mod.item_3d", 3),
        new ModEntry(41, "paraguacraft.menu.mod.low_fire", 2),
        new ModEntry(43, "paraguacraft.menu.mod.opponent_ping", 2),
        new ModEntry(44, "paraguacraft.menu.mod.quick_play", 7),
        new ModEntry(45, "paraguacraft.menu.mod.chat_triggers", 7),
        new ModEntry(46, "paraguacraft.menu.mod.freelook", 2),
        new ModEntry(47, "paraguacraft.menu.mod.reach_display", 2),
        new ModEntry(48, "paraguacraft.menu.mod.combo_counter", 2),
        new ModEntry(49, "paraguacraft.menu.mod.item_physics", 3),
        new ModEntry(50, "paraguacraft.menu.mod.hide_titles", 2),
        new ModEntry(51, "paraguacraft.menu.mod.idle_fps_min", 6),
        new ModEntry(52, "paraguacraft.menu.mod.idle_fps_unfocus", 6),
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

        fr.drawStringWithShadow(ModLang.format("paraguacraft.menu.hint"), panelX + 12, panelY + panelH - 14, UiTheme.TEXT_DIM);
        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    private void drawSidebar(int x, int y, int h, int mouseX, int mouseY) {
        FontRenderer fr = fr();
        Gui.drawRect(x, y, x + SIDEBAR, y + h, 0xDD080A10);
        fr.drawStringWithShadow(ModLang.format("paraguacraft.menu.brand"), x + 14, y + 16, UiTheme.ACCENT);
        fr.drawStringWithShadow(ModLang.format("paraguacraft.menu.subtitle"), x + 14, y + 38, UiTheme.TEXT_DIM);

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
                ModLang.format("paraguacraft.menu.cat." + CATEGORY_IDS[i]),
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
        String placeholder = ModLang.format("paraguacraft.menu.search.placeholder");
        String shown = searchQuery.isEmpty() ? placeholder : searchQuery;
        int color = searchQuery.isEmpty() ? UiTheme.TEXT_DIM : UiTheme.TEXT;
        fr.drawStringWithShadow(shown + (searchFocused && (System.currentTimeMillis() / 500) % 2 == 0 ? "_" : ""), searchX + 8, searchY + 7, color);
    }

    private void drawModGrid(int x, int y, int w, int h, int mouseX, int mouseY) {
        List<ModEntry> visible = filteredMods();
        int col = Math.max(1, (w + GAP) / (CARD_W + GAP));
        int rows = (visible.size() + col - 1) / col;
        int contentH = rows > 0 ? rows * (CARD_H + GAP) + 8 : 0;
        int maxScroll = Math.max(0, contentH - h);
        int hash = visible.size() * 31 + selectedCategory * 17 + searchQuery.hashCode();
        if (hash != lastVisibleHash) {
            lastVisibleHash = hash;
            scrollOffset = 0f;
        }
        scrollOffset = Math.max(0f, Math.min(maxScroll, scrollOffset));

        ScaledResolution sr = new ScaledResolution(mc);
        int factor = sr.getScaleFactor();
        GL11.glEnable(GL11.GL_SCISSOR_TEST);
        GL11.glScissor(x * factor, org.lwjgl.opengl.Display.getHeight() - (y + h) * factor, w * factor, h * factor);
        GlStateManager.disableDepth();

        int cx = 0;
        int cy = 0;
        for (ModEntry mod : visible) {
            int cardX = x + 8 + cx * (CARD_W + GAP);
            int cardY = y + 8 + cy * (CARD_H + GAP) - (int) scrollOffset;
            if (cardY + CARD_H >= y && cardY <= y + h) {
                drawModCard(mod, cardX, cardY, getModState(mod.id), mouseX, mouseY);
            }
            cx++;
            if (cx >= col) {
                cx = 0;
                cy++;
            }
        }

        GL11.glDisable(GL11.GL_SCISSOR_TEST);
        GlStateManager.enableDepth();

        if (visible.isEmpty()) {
            FontRenderer fr = fr();
            String empty = ModLang.format("paraguacraft.menu.empty");
            fr.drawStringWithShadow(empty, x + w / 2 - fr.getStringWidth(empty) / 2, y + h / 2, UiTheme.TEXT_DIM);
        }
    }

    private void drawModCard(ModEntry mod, int x, int y, boolean enabled, int mouseX, int mouseY) {
        FontRenderer fr = fr();
        boolean isScreen = mod.id == 16 || mod.id == 33 || mod.id == 34 || mod.id == 44;
        boolean hover = mouseX >= x && mouseX <= x + CARD_W && mouseY >= y && mouseY <= y + CARD_H;
        float target = isScreen ? 1f : (enabled ? 1f : 0f);
        float anim = toggleAnim.containsKey(mod.id) ? toggleAnim.get(mod.id) : target;
        anim = UiEasing.approach(anim, target, 0.22f);
        toggleAnim.put(mod.id, anim);

        int bg = hover ? 0xCC161A24 : 0xAA101218;
        Gui.drawRect(x, y, x + CARD_W, y + CARD_H, bg);
        Gui.drawRect(x, y, x + CARD_W, y + 1, 0x28FFFFFF);

        Gui.drawRect(x + 10, y + 10, x + 42, y + 42, 0x3300E5FF);
        mc.getTextureManager().bindTexture(MOD_ICON);
        GlStateManager.color(1f, 1f, 1f, 1f);
        Gui.drawModalRectWithCustomSizedTexture(x + 10, y + 10, 0, 0, 32, 32, 32, 32);
        String display = modDisplayName(mod.id);
        fr.drawStringWithShadow(display, x + 50, y + 16, isScreen || enabled ? UiTheme.TEXT : UiTheme.TEXT_DIM);

        int toggleY = y + CARD_H - 22;
        if (isScreen) {
            String openLbl = ModLang.format("paraguacraft.menu.open");
            Gui.drawRect(x + 8, toggleY, x + CARD_W - 8, toggleY + 16, hover ? UiTheme.ACCENT : 0xFF226688);
            fr.drawStringWithShadow(openLbl, x + CARD_W / 2 - fr.getStringWidth(openLbl) / 2, toggleY + 4, 0xFFFFFF);
        } else if (mod.id == 9 || mod.id == 36 || mod.id == 38 || mod.id == 45) {
            int half = (CARD_W - 20) / 2;
            int optX = x + 8;
            int togX = x + 8 + half + 4;
            boolean hoverOpt = mouseX >= optX && mouseX <= optX + half && mouseY >= toggleY && mouseY <= toggleY + 16;
            String opts = ModLang.format("paraguacraft.menu.options");
            Gui.drawRect(optX, toggleY, optX + half, toggleY + 16, hoverOpt ? 0xFF334455 : 0xFF223344);
            fr.drawStringWithShadow(opts, optX + half / 2 - fr.getStringWidth(opts) / 2, toggleY + 4, 0xFFFFFF);
            int toggleColor = lerpColor(0xFFCC4444, 0xFF22CC66, UiEasing.easeOutCubic(anim));
            Gui.drawRect(togX, toggleY, x + CARD_W - 8, toggleY + 16, toggleColor);
            String stateLabel = modStateLabel(mod.id, enabled);
            fr.drawStringWithShadow(stateLabel, togX + half / 2 - fr.getStringWidth(stateLabel) / 2, toggleY + 4, 0xFFFFFF);
        } else {
            int toggleColor = lerpColor(0xFFCC4444, 0xFF22CC66, UiEasing.easeOutCubic(anim));
            Gui.drawRect(x + 8, toggleY, x + CARD_W - 8, toggleY + 16, toggleColor);
            String stateLabel = modStateLabel(mod.id, enabled);
            fr.drawStringWithShadow(stateLabel, x + CARD_W / 2 - fr.getStringWidth(stateLabel) / 2, toggleY + 4, 0xFFFFFF);
        }
    }

    private String modDisplayName(int id) {
        if (id == 23) {
            return ModLang.format("paraguacraft.menu.particles.prefix", PerformanceConfig.particleMode.getLabel());
        }
        if (id == 32) {
            return ModLang.format("paraguacraft.menu.hw.prefix", com.paraguacraft.pvp.core.HardwarePreset.getDetectedTier().name());
        }
        for (ModEntry mod : ALL_MODS) {
            if (mod.id == id) {
                return ModLang.format(mod.langKey);
            }
        }
        return "?";
    }

    private String modStateLabel(int id, boolean enabled) {
        if (id == 23) {
            return PerformanceConfig.particleMode.getLabel().toUpperCase();
        }
        return ModLang.format(enabled ? "paraguacraft.menu.on" : "paraguacraft.menu.off");
    }

    private List<ModEntry> filteredMods() {
        List<ModEntry> out = new ArrayList<ModEntry>();
        for (ModEntry mod : ALL_MODS) {
            if (selectedCategory > 0 && mod.category != selectedCategory) {
                continue;
            }
            if (!searchQuery.isEmpty()) {
                String label = modDisplayName(mod.id);
                if (!TextUtil.containsIgnoreCase(ModLang.format(mod.langKey), searchQuery)
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
            int cardY = gridY + 8 + cy * (CARD_H + GAP) - (int) scrollOffset;
            if (cardY + CARD_H >= gridY && cardY <= gridY + panelH - TOPBAR - 12
                && mouseX >= cardX && mouseX <= cardX + CARD_W && mouseY >= cardY && mouseY <= cardY + CARD_H) {
                if (mod.id == 9 || mod.id == 36 || mod.id == 38 || mod.id == 45) {
                    int toggleY = cardY + CARD_H - 22;
                    int half = (CARD_W - 20) / 2;
                    int optX = cardX + 8;
                    int togX = cardX + 8 + half + 4;
                    if (mouseY >= toggleY && mouseY <= toggleY + 16) {
                        if (mouseX >= optX && mouseX <= optX + half) {
                            if (mod.id == 9) {
                                mc.displayGuiScreen(new GuiScoreboardOptions());
                            } else if (mod.id == 36) {
                                mc.displayGuiScreen(new GuiMusicHudOptions());
                            } else if (mod.id == 45) {
                                mc.displayGuiScreen(new GuiChatTriggersOptions());
                            } else {
                                mc.displayGuiScreen(new GuiBwResourcesOptions());
                            }
                            return;
                        }
                        if (mouseX >= togX && mouseX <= cardX + CARD_W - 8) {
                            toggleMod(mod.id);
                            ModConfig.save();
                            return;
                        }
                    }
                    return;
                }
                if (mod.id == 16) {
                    mc.displayGuiScreen(new GuiResourcePacks());
                } else if (mod.id == 33) {
                    mc.displayGuiScreen(new GuiModProfiles());
                } else if (mod.id == 34) {
                    mc.displayGuiScreen(new GuiKeybinds());
                } else if (mod.id == 44) {
                    mc.displayGuiScreen(new GuiHypixelQuickPlay());
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
    public void handleMouseInput() throws IOException {
        super.handleMouseInput();
        int wheel = Mouse.getEventDWheel();
        if (wheel == 0) {
            return;
        }
        int panelX = Math.max(8, width / 2 - 420);
        int panelY = 24;
        int panelW = Math.min(width - 16, 840);
        int panelH = height - 48;
        int gridX = panelX + SIDEBAR;
        int gridY = panelY + TOPBAR;
        int gridW = panelW - SIDEBAR - 12;
        int gridH = panelH - TOPBAR - 12;
        int mx = Mouse.getEventX() * width / mc.displayWidth;
        int my = height - Mouse.getEventY() * height / mc.displayHeight - 1;
        if (mx < gridX || mx > gridX + gridW || my < gridY || my > gridY + gridH) {
            return;
        }
        List<ModEntry> visible = filteredMods();
        int col = Math.max(1, (gridW + GAP) / (CARD_W + GAP));
        int rows = (visible.size() + col - 1) / col;
        int contentH = rows > 0 ? rows * (CARD_H + GAP) + 8 : 0;
        int maxScroll = Math.max(0, contentH - gridH);
        scrollOffset = Math.max(0f, Math.min(maxScroll, scrollOffset - wheel * 0.25f));
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
            case 9: return ModConfig.scoreboardEnabled;
            case 10: return ModConfig.toggleSneak;
            case 53: return ModConfig.toggleSprintActive;
            case 54: return ModConfig.toggleSprintLegacyActive;
            case 11: return ModConfig.dynamicFov;
            case 12: return ModConfig.showHeldItem;
            case 13: return ModConfig.windowedFullscreen;
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
            case 35: return ModConfig.showHardwareHud;
            case 36: return ModConfig.showMusicHud;
            case 37: return ModConfig.showTntCountdown;
            case 38: return ModConfig.showBedwarsResources;
            case 39: return ModConfig.forceItem3d;
            case 41: return ModConfig.lowFire;
            case 43: return ModConfig.showOpponentPing;
            case 44: return true;
            case 45: return ModConfig.chatTriggers;
            case 46: return ModConfig.freelookEnabled;
            case 47: return ModConfig.reachDisplay;
            case 48: return ModConfig.comboCounter;
            case 49: return ModConfig.itemPhysics;
            case 50: return ModConfig.hideTitles;
            case 51: return PerformanceConfig.reduceFpsWhenMinimized;
            case 52: return PerformanceConfig.reduceFpsWhenUnfocused;
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
            case 9: ModConfig.scoreboardEnabled = !ModConfig.scoreboardEnabled; break;
            case 10: ModConfig.toggleSneak = !ModConfig.toggleSneak; ModConfig.isSneakingToggled = false; break;
            case 53: ModConfig.toggleSprintActive = !ModConfig.toggleSprintActive; break;
            case 54: ModConfig.toggleSprintLegacyActive = !ModConfig.toggleSprintLegacyActive; break;
            case 11: ModConfig.dynamicFov = !ModConfig.dynamicFov; break;
            case 12: ModConfig.showHeldItem = !ModConfig.showHeldItem; break;
            case 13:
                ModConfig.windowedFullscreen = !ModConfig.windowedFullscreen;
                ModConfig.save();
                if (this.mc != null
                        && ModConfig.windowedFullscreen != ModConfig.windowedActive) {
                    // Activar => entrar en borderless; desactivar => salir.
                    this.mc.toggleFullscreen();
                }
                break;
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
            case 35: ModConfig.showHardwareHud = !ModConfig.showHardwareHud; break;
            case 36: ModConfig.showMusicHud = !ModConfig.showMusicHud; break;
            case 37: ModConfig.showTntCountdown = !ModConfig.showTntCountdown; break;
            case 38: ModConfig.showBedwarsResources = !ModConfig.showBedwarsResources; break;
            case 39: ModConfig.forceItem3d = !ModConfig.forceItem3d; break;
            case 41: ModConfig.lowFire = !ModConfig.lowFire; break;
            case 43: ModConfig.showOpponentPing = !ModConfig.showOpponentPing; break;
            case 45: ModConfig.chatTriggers = !ModConfig.chatTriggers; break;
            case 46: ModConfig.freelookEnabled = !ModConfig.freelookEnabled; break;
            case 47: ModConfig.reachDisplay = !ModConfig.reachDisplay; break;
            case 48: ModConfig.comboCounter = !ModConfig.comboCounter; break;
            case 49: ModConfig.itemPhysics = !ModConfig.itemPhysics; break;
            case 50: ModConfig.hideTitles = !ModConfig.hideTitles; break;
            case 51:
                PerformanceConfig.reduceFpsWhenMinimized = !PerformanceConfig.reduceFpsWhenMinimized;
                ModConfig.save();
                break;
            case 52:
                PerformanceConfig.reduceFpsWhenUnfocused = !PerformanceConfig.reduceFpsWhenUnfocused;
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
