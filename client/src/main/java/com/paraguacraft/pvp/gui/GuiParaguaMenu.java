package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.core.GameModeDetector;
import com.paraguacraft.pvp.core.ModLang;
import com.paraguacraft.pvp.core.OptifinePreset;
import com.paraguacraft.pvp.core.PerformanceConfig;
import com.paraguacraft.pvp.core.ServerContext;
import com.paraguacraft.pvp.gui.theme.TextUtil;
import com.paraguacraft.pvp.gui.theme.UiEasing;
import com.paraguacraft.pvp.gui.theme.UiTheme;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.gui.ScaledResolution;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.util.ResourceLocation;
import org.lwjgl.input.Keyboard;
import org.lwjgl.input.Mouse;
import org.lwjgl.opengl.GL11;

import java.io.IOException;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;

/** Hub Right Shift: overlay compacto, pestañas Mods / Ajustes / Packs. */
public class GuiParaguaMenu extends GuiScreen {

    private static final ResourceLocation MOD_ICON = new ResourceLocation("paraguacraft", "textures/gui/logo.png");

    private static final int TOPBAR = 78;
    private static final int FOOTER = 30;
    private static final int CARD_W = 168;
    private static final int CARD_H = 72;
    private static final int GAP = 12;
    private static final int TAB_MODS = 0;
    private static final int TAB_SETTINGS = 1;
    private static final int TAB_PACKS = 2;
    private static final String[] TAB_IDS = {"mods", "settings", "packs"};

    private int selectedTab;
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
        new ModEntry(60, "paraguacraft.menu.mod.fps", 1),
        new ModEntry(1, "paraguacraft.menu.mod.ping", 1),
        new ModEntry(2, "paraguacraft.menu.mod.cps", 1),
        new ModEntry(3, "paraguacraft.menu.mod.keystrokes", 1),
        new ModEntry(63, "paraguacraft.menu.mod.saturation", 1),
        new ModEntry(5, "paraguacraft.menu.mod.coords", 1),
        new ModEntry(6, "paraguacraft.menu.mod.armor", 1),
        new ModEntry(8, "paraguacraft.menu.mod.potions", 1),
        new ModEntry(12, "paraguacraft.menu.mod.held_item", 1),
        new ModEntry(14, "paraguacraft.menu.mod.server_hud", 4),
        new ModEntry(15, "paraguacraft.menu.mod.compass", 1),
        new ModEntry(73, "paraguacraft.menu.mod.height_limit", 1),
        new ModEntry(74, "paraguacraft.menu.mod.crosshair", 2),
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
        new ModEntry(66, "paraguacraft.menu.mod.motion_blur", 3),
        new ModEntry(20, "paraguacraft.menu.mod.old_anim", 3),
        new ModEntry(19, "paraguacraft.menu.mod.boost_fps", 6),
        new ModEntry(61, "paraguacraft.menu.mod.entity_cull", 6),
        new ModEntry(23, "paraguacraft.menu.mod.particles", 6),
        new ModEntry(26, "paraguacraft.menu.mod.memory_clean", 6),
        new ModEntry(27, "paraguacraft.menu.mod.optifine_preset", 6),
        new ModEntry(31, "paraguacraft.menu.mod.skip_combat_fx", 6),
        new ModEntry(32, "paraguacraft.menu.mod.hw_preset", 6),
        new ModEntry(33, "paraguacraft.menu.mod.profiles", 3),
        new ModEntry(34, "paraguacraft.menu.mod.keybinds", 3),
        new ModEntry(81, "paraguacraft.menu.mod.auto_profiles", 3),
        new ModEntry(82, "paraguacraft.menu.mod.profile", 3),
        new ModEntry(35, "paraguacraft.menu.mod.hardware_hud", 1),
        new ModEntry(36, "paraguacraft.menu.mod.music_hud", 1),
        new ModEntry(37, "paraguacraft.menu.mod.tnt_countdown", 2),
        new ModEntry(38, "paraguacraft.menu.mod.bw_resources", 2),
        new ModEntry(39, "paraguacraft.menu.mod.item_3d", 3),
        new ModEntry(62, "paraguacraft.menu.mod.waypoints", 2),
        new ModEntry(64, "paraguacraft.menu.mod.item_tracker", 2),
        new ModEntry(65, "paraguacraft.menu.mod.watermark", 4),
        new ModEntry(41, "paraguacraft.menu.mod.low_fire", 2),
        new ModEntry(43, "paraguacraft.menu.mod.opponent_ping", 2),
        new ModEntry(44, "paraguacraft.menu.mod.quick_play", 7),
        new ModEntry(45, "paraguacraft.menu.mod.chat_triggers", 7),
        new ModEntry(46, "paraguacraft.menu.mod.freelook", 2),
        new ModEntry(47, "paraguacraft.menu.mod.reach_display", 2),
        new ModEntry(48, "paraguacraft.menu.mod.combo_counter", 2),
        new ModEntry(49, "paraguacraft.menu.mod.item_physics", 3),
        new ModEntry(50, "paraguacraft.menu.mod.hide_titles", 2),
        new ModEntry(67, "paraguacraft.menu.mod.chat", 2),
        new ModEntry(68, "paraguacraft.menu.mod.tab_editor", 2),
        new ModEntry(70, "paraguacraft.menu.mod.hit_color", 2),
        new ModEntry(71, "paraguacraft.menu.mod.pack_hud", 1),
        new ModEntry(72, "paraguacraft.menu.mod.gui_scale", 3),
    };

    @Override
    public void initGui() {
        searchFocused = selectedTab == TAB_MODS;
    }

    private FontRenderer fr() {
        return this.fontRendererObj;
    }

    private int[] panelGeom(float s) {
        int sw = (int) (width / s);
        int sh = (int) (height / s);
        int panelW = Math.min(sw - 24, 720);
        int panelH = Math.min(sh - 28, 400);
        int panelX = (sw - panelW) / 2;
        int panelY = Math.max(8, (sh - panelH) / 2);
        return new int[] {panelX, panelY, panelW, panelH};
    }

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        drawRect(0, 0, width, height, 0x44000000);

        float s = ModConfig.uiScaleFactor();
        int mx = (int) (mouseX / s);
        int my = (int) (mouseY / s);

        GlStateManager.pushMatrix();
        if (s != 1.0f) {
            GlStateManager.scale(s, s, 1.0f);
        }

        int[] g = panelGeom(s);
        int panelX = g[0];
        int panelY = g[1];
        int panelW = g[2];
        int panelH = g[3];
        Gui.drawRect(panelX, panelY, panelX + panelW, panelY + panelH, 0xCC0A0C14);
        Gui.drawRect(panelX, panelY, panelX + panelW, panelY + 1, 0x33FFFFFF);
        Gui.drawRect(panelX, panelY + panelH - 1, panelX + panelW, panelY + panelH, 0x22FFFFFF);

        drawHeader(panelX, panelY, panelW, mx, my);
        drawModGrid(panelX + 8, panelY + TOPBAR, panelW - 16, panelH - TOPBAR - FOOTER, mx, my);
        drawFooter(panelX, panelY, panelW, panelH, mx, my);

        GlStateManager.popMatrix();
        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    private void drawHeader(int x, int y, int w, int mouseX, int mouseY) {
        FontRenderer fr = fr();
        Gui.drawRect(x, y, x + w, y + TOPBAR, 0x88101018);
        fr.drawStringWithShadow(ModLang.format("paraguacraft.menu.brand"), x + 12, y + 8, UiTheme.ACCENT);

        String chip = ServerContext.chip();
        int chipW = fr.getStringWidth(chip) + 12;
        int chipX = x + w - chipW - 12;
        Gui.drawRect(chipX, y + 6, chipX + chipW, y + 20, 0xAA123040);
        fr.drawStringWithShadow(chip, chipX + 6, y + 9, UiTheme.ACCENT);

        int tabY = y + 26;
        int tabW = 78;
        for (int i = 0; i < TAB_IDS.length; i++) {
            int tx = x + 12 + i * (tabW + 6);
            boolean selected = i == selectedTab;
            boolean hover = mouseX >= tx && mouseX <= tx + tabW && mouseY >= tabY && mouseY <= tabY + 18;
            Gui.drawRect(tx, tabY, tx + tabW, tabY + 18, selected ? 0x4400E5FF : (hover ? 0x22FFFFFF : 0x22000000));
            String label = ModLang.format("paraguacraft.menu.tab." + TAB_IDS[i]);
            fr.drawStringWithShadow(label, tx + tabW / 2 - fr.getStringWidth(label) / 2, tabY + 5,
                selected ? UiTheme.TEXT : UiTheme.TEXT_DIM);
        }

        if (selectedTab == TAB_MODS) {
            int searchX = x + 12;
            int searchY = y + 50;
            int searchW = Math.min(280, w - 130);
            boolean hoverSearch = mouseX >= searchX && mouseX <= searchX + searchW && mouseY >= searchY && mouseY <= searchY + 20;
            Gui.drawRect(searchX, searchY, searchX + searchW, searchY + 20, hoverSearch || searchFocused ? 0xAA121820 : 0x88000000);
            if (searchFocused) {
                Gui.drawRect(searchX, searchY + 19, searchX + searchW, searchY + 20, UiTheme.ACCENT);
            }
            String placeholder = ModLang.format("paraguacraft.menu.search.placeholder");
            String shown = searchQuery.isEmpty() ? placeholder : searchQuery;
            int color = searchQuery.isEmpty() ? UiTheme.TEXT_DIM : UiTheme.TEXT;
            fr.drawStringWithShadow(shown + (searchFocused && (System.currentTimeMillis() / 500) % 2 == 0 ? "_" : ""),
                searchX + 8, searchY + 6, color);

            int scaleX = searchX + searchW + 8;
            int scaleW = 92;
            boolean hoverScale = mouseX >= scaleX && mouseX <= scaleX + scaleW && mouseY >= searchY && mouseY <= searchY + 20;
            Gui.drawRect(scaleX, searchY, scaleX + scaleW, searchY + 20, hoverScale ? 0xAA123040 : 0x88000000);
            String scaleLbl = "UI " + ModConfig.uiScaleLabel();
            fr.drawStringWithShadow(scaleLbl, scaleX + (scaleW - fr.getStringWidth(scaleLbl)) / 2, searchY + 6, UiTheme.ACCENT);
        }
    }

    private void drawFooter(int x, int y, int w, int h, int mouseX, int mouseY) {
        FontRenderer fr = fr();
        int fy = y + h - FOOTER;
        Gui.drawRect(x, fy, x + w, y + h, 0xAA080A10);
        int btnW = 110;
        boolean hoverHud = mouseX >= x + 10 && mouseX <= x + 10 + btnW && mouseY >= fy + 6 && mouseY <= fy + 24;
        Gui.drawRect(x + 10, fy + 6, x + 10 + btnW, fy + 24, hoverHud ? UiTheme.ACCENT : 0xFF226688);
        String hudLbl = ModLang.format("paraguacraft.menu.edit_hud");
        fr.drawStringWithShadow(hudLbl, x + 10 + btnW / 2 - fr.getStringWidth(hudLbl) / 2, fy + 11, 0xFFFFFF);
        fr.drawStringWithShadow(ModLang.format("paraguacraft.menu.hint"), x + 128, fy + 11, UiTheme.TEXT_DIM);
    }

    private void drawModGrid(int x, int y, int w, int h, int mouseX, int mouseY) {
        List<ModEntry> visible = filteredMods();
        int rec = recommendedCount();
        int headerH = rec > 0 ? 14 : 0;
        int col = Math.max(1, (w + GAP) / (CARD_W + GAP));
        int rows = (visible.size() + col - 1) / col;
        int contentH = (rows > 0 ? rows * (CARD_H + GAP) + 8 : 0) + headerH;
        int maxScroll = Math.max(0, contentH - h);
        int hash = visible.size() * 31 + selectedTab * 17 + searchQuery.hashCode() + rec * 7;
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

        if (rec > 0) {
            FontRenderer fr = fr();
            fr.drawStringWithShadow(ModLang.format("paraguacraft.menu.for_mode"), x + 8, y + 2 - (int) scrollOffset, UiTheme.ACCENT);
        }

        int cx = 0;
        int cy = 0;
        for (ModEntry mod : visible) {
            int cardX = x + 8 + cx * (CARD_W + GAP);
            int cardY = y + 8 + headerH + cy * (CARD_H + GAP) - (int) scrollOffset;
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
        boolean isScreen = isOpenCard(mod.id);
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
        } else if (hasOptions(mod.id)) {
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
        if (id == 74) {
            return ModLang.format("paraguacraft.menu.mod.crosshair") + ": " + ModConfig.crosshairModeLabel();
        }
        if (id == 82) {
            return ModLang.format("paraguacraft.menu.mod.profile") + ": " + GameModeDetector.overrideLabel();
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
        if (id == 74) {
            return ModConfig.crosshairModeLabel();
        }
        if (id == 82) {
            return GameModeDetector.overrideLabel();
        }
        return ModLang.format(enabled ? "paraguacraft.menu.on" : "paraguacraft.menu.off");
    }

    private static boolean isOpenCard(int id) {
        return id == 16 || id == 33 || id == 34 || id == 44;
    }

    private static int[] recommendedIds() {
        GameModeDetector.Mode mode = GameModeDetector.current();
        switch (mode) {
            case BEDWARS:
                return new int[] {38, 73, 37, 6, 15};
            case SKYWARS:
            case LUCKY_ISLANDS:
                return new int[] {6, 8, 12, 47};
            case DUELS:
                return new int[] {47, 48, 70, 6, 43};
            case UHC:
                return new int[] {5, 6, 8, 15};
            case LOBBY:
                return new int[] {14, 16, 44};
            default:
                return new int[] {47, 48, 6, 4};
        }
    }

    private int recommendedCount() {
        if (selectedTab != TAB_MODS || !searchQuery.isEmpty()) {
            return 0;
        }
        return recommendedIds().length;
    }

    private List<ModEntry> filteredMods() {
        if (selectedTab == TAB_SETTINGS) {
            return entriesByIds(new int[] {74, 81, 82, 13, 66, 72, 11, 63, 34, 33});
        }
        if (selectedTab == TAB_PACKS) {
            return entriesByIds(new int[] {16, 71});
        }
        List<ModEntry> out = new ArrayList<ModEntry>();
        Set<Integer> seen = new HashSet<Integer>();
        if (searchQuery.isEmpty()) {
            int[] rec = recommendedIds();
            for (int i = 0; i < rec.length; i++) {
                ModEntry e = entryById(rec[i]);
                if (e != null) {
                    out.add(e);
                    seen.add(Integer.valueOf(e.id));
                }
            }
        }
        for (ModEntry mod : ALL_MODS) {
            if (isSettingsOnly(mod.id) || mod.id == 16) {
                continue;
            }
            if (seen.contains(Integer.valueOf(mod.id))) {
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

    private static boolean isSettingsOnly(int id) {
        return id == 33 || id == 34 || id == 81 || id == 82 || id == 13 || id == 66 || id == 72;
    }

    private List<ModEntry> entriesByIds(int[] ids) {
        List<ModEntry> out = new ArrayList<ModEntry>();
        for (int i = 0; i < ids.length; i++) {
            ModEntry e = entryById(ids[i]);
            if (e != null) {
                out.add(e);
            }
        }
        return out;
    }

    private ModEntry entryById(int id) {
        for (ModEntry mod : ALL_MODS) {
            if (mod.id == id) {
                return mod;
            }
        }
        return null;
    }

    @Override
    protected void mouseClicked(int mouseX, int mouseY, int mouseButton) throws IOException {
        if (mouseButton != 0) {
            return;
        }
        float s = ModConfig.uiScaleFactor();
        int mx = (int) (mouseX / s);
        int my = (int) (mouseY / s);
        int[] g = panelGeom(s);
        int panelX = g[0];
        int panelY = g[1];
        int panelW = g[2];
        int panelH = g[3];

        int fy = panelY + panelH - FOOTER;
        if (mx >= panelX + 10 && mx <= panelX + 120 && my >= fy + 6 && my <= fy + 24) {
            mc.displayGuiScreen(new GuiEditHUD());
            return;
        }

        int tabY = panelY + 26;
        int tabW = 78;
        for (int i = 0; i < TAB_IDS.length; i++) {
            int tx = panelX + 12 + i * (tabW + 6);
            if (mx >= tx && mx <= tx + tabW && my >= tabY && my <= tabY + 18) {
                selectedTab = i;
                searchFocused = selectedTab == TAB_MODS;
                scrollOffset = 0f;
                return;
            }
        }

        if (selectedTab == TAB_MODS) {
            int searchX = panelX + 12;
            int searchY = panelY + 50;
            int searchW = Math.min(280, panelW - 130);
            int scaleX = searchX + searchW + 8;
            int scaleW = 92;
            if (mx >= scaleX && mx <= scaleX + scaleW && my >= searchY && my <= searchY + 20) {
                ModConfig.cycleUiScale();
                ModConfig.save();
                return;
            }
            if (mx >= searchX && mx <= searchX + searchW && my >= searchY && my <= searchY + 20) {
                searchFocused = true;
                return;
            }
        }
        searchFocused = false;

        List<ModEntry> visible = filteredMods();
        int rec = recommendedCount();
        int headerH = rec > 0 ? 14 : 0;
        int gridX = panelX + 8;
        int gridY = panelY + TOPBAR;
        int gridW = panelW - 16;
        int col = Math.max(1, (gridW + GAP) / (CARD_W + GAP));
        int cx = 0;
        int cy = 0;
        for (ModEntry mod : visible) {
            int cardX = gridX + 8 + cx * (CARD_W + GAP);
            int cardY = gridY + 8 + headerH + cy * (CARD_H + GAP) - (int) scrollOffset;
            if (cardY + CARD_H >= gridY && cardY <= gridY + panelH - TOPBAR - FOOTER
                && mx >= cardX && mx <= cardX + CARD_W && my >= cardY && my <= cardY + CARD_H) {
                handleCardClick(mod, cardX, cardY, mx, my);
                return;
            }
            cx++;
            if (cx >= col) {
                cx = 0;
                cy++;
            }
        }
    }

    private void handleCardClick(ModEntry mod, int cardX, int cardY, int mx, int my) {
        if (hasOptions(mod.id)) {
            int toggleY = cardY + CARD_H - 22;
            int half = (CARD_W - 20) / 2;
            int optX = cardX + 8;
            int togX = cardX + 8 + half + 4;
            if (my >= toggleY && my <= toggleY + 16) {
                if (mx >= optX && mx <= optX + half) {
                    openOptions(mod.id);
                    return;
                }
                if (mx >= togX && mx <= cardX + CARD_W - 8) {
                    toggleMod(mod.id);
                    ModConfig.save();
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
    }

    private void openOptions(int id) {
        if (id == 3) {
            mc.displayGuiScreen(GuiSubmodOptions.keystrokes());
        } else if (id == 6) {
            mc.displayGuiScreen(GuiSubmodOptions.armor());
        } else if (id == 9) {
            mc.displayGuiScreen(new GuiScoreboardOptions());
        } else if (id == 17) {
            mc.displayGuiScreen(GuiSubmodOptions.nametag());
        } else if (id == 23) {
            mc.displayGuiScreen(GuiSubmodOptions.particles());
        } else if (id == 36) {
            mc.displayGuiScreen(new GuiMusicHudOptions());
        } else if (id == 66) {
            mc.displayGuiScreen(new GuiMotionBlurOptions());
        } else if (id == 37) {
            mc.displayGuiScreen(GuiSubmodOptions.tnt());
        } else if (id == 38) {
            mc.displayGuiScreen(GuiSubmodOptions.bedwars());
        } else if (id == 45) {
            mc.displayGuiScreen(new GuiChatTriggersOptions());
        } else if (id == 47) {
            mc.displayGuiScreen(GuiSubmodOptions.reach());
        } else if (id == 60) {
            mc.displayGuiScreen(GuiSubmodOptions.fps());
        } else if (id == 64) {
            mc.displayGuiScreen(GuiSubmodOptions.items());
        } else if (id == 67) {
            mc.displayGuiScreen(GuiSubmodOptions.chat());
        } else if (id == 68) {
            mc.displayGuiScreen(GuiSubmodOptions.tab());
        } else if (id == 70) {
            mc.displayGuiScreen(GuiSubmodOptions.hitColor());
        } else if (id == 71) {
            mc.displayGuiScreen(GuiSubmodOptions.packHud());
        } else if (id == 72) {
            mc.displayGuiScreen(GuiSubmodOptions.guiScale());
        } else if (id == 73) {
            mc.displayGuiScreen(GuiSubmodOptions.heightLimit());
        } else {
            mc.displayGuiScreen(GuiSubmodOptions.entity());
        }
    }

    @Override
    public void handleMouseInput() throws IOException {
        super.handleMouseInput();
        int wheel = Mouse.getEventDWheel();
        if (wheel == 0) {
            return;
        }
        float s = ModConfig.uiScaleFactor();
        int[] g = panelGeom(s);
        int panelX = g[0];
        int panelY = g[1];
        int panelW = g[2];
        int panelH = g[3];
        int gridX = panelX + 8;
        int gridY = panelY + TOPBAR;
        int gridW = panelW - 16;
        int gridH = panelH - TOPBAR - FOOTER;
        int mx = (int) ((Mouse.getEventX() * width / mc.displayWidth) / s);
        int my = (int) ((height - Mouse.getEventY() * height / mc.displayHeight - 1) / s);
        if (mx < gridX || mx > gridX + gridW || my < gridY || my > gridY + gridH) {
            return;
        }
        List<ModEntry> visible = filteredMods();
        int rec = recommendedCount();
        int headerH = rec > 0 ? 14 : 0;
        int col = Math.max(1, (gridW + GAP) / (CARD_W + GAP));
        int rows = (visible.size() + col - 1) / col;
        int contentH = (rows > 0 ? rows * (CARD_H + GAP) + 8 : 0) + headerH;
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
            case 8: return ModConfig.showPotions;
            case 9: return ModConfig.scoreboardEnabled;
            case 10: return ModConfig.toggleSneak;
            case 53: return ModConfig.toggleSprintActive;
            case 54: return ModConfig.toggleSprintLegacyActive;
            case 11: return ModConfig.dynamicFov;
            case 12: return ModConfig.showHeldItem;
            case 13: return ModConfig.windowedFullscreen;
            case 66: return ModConfig.motionBlurEnabled;
            case 14: return ModConfig.showServerHUD;
            case 15: return ModConfig.showCompass;
            case 16: return true;
            case 17: return ModConfig.showNametagLogo;
            case 18: return ModConfig.showNametagLogoOthers;
            case 19: return PerformanceConfig.boostFps;
            case 20: return PerformanceConfig.oldAnimations;
            case 23: return PerformanceConfig.particleMode != PerformanceConfig.ParticleMode.OFF;
            case 26: return PerformanceConfig.memoryCleanupOnWorldChange;
            case 27: return PerformanceConfig.applyVanillaPreset;
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
            case 60: return ModConfig.showFPS;
            case 61: return PerformanceConfig.entityCull;
            case 62: return ModConfig.showWaypoints;
            case 63: return ModConfig.showSaturation;
            case 64: return ModConfig.itemTracker2d || ModConfig.itemTracker3d;
            case 65: return ModConfig.showWatermark;
            case 67: return ModConfig.chatUnlimited;
            case 68: return ModConfig.tabEditor;
            case 70: return ModConfig.hitColorEnabled;
            case 71: return ModConfig.showPackHud;
            case 72: return ModConfig.hotbarScale != 100 || ModConfig.inventoryScale != 100;
            case 73: return ModConfig.showHeightLimit;
            case 74: return ModConfig.crosshairMode != 0;
            case 81: return ModConfig.autoGameModeProfiles;
            case 82: return GameModeDetector.isManualOverride();
            default: return false;
        }
    }

    private static boolean hasOptions(int id) {
        return id == 3 || id == 6 || id == 9 || id == 17 || id == 23 || id == 36 || id == 37 || id == 38
            || id == 45 || id == 47 || id == 60 || id == 61 || id == 64 || id == 66
            || id == 67 || id == 68 || id == 70 || id == 71 || id == 72 || id == 73;
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
            case 60: ModConfig.showFPS = !ModConfig.showFPS; break;
            case 61:
                PerformanceConfig.entityCull = !PerformanceConfig.entityCull;
                ModConfig.save();
                break;
            case 23:
                PerformanceConfig.cycleParticleMode();
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
            case 62: ModConfig.showWaypoints = !ModConfig.showWaypoints; break;
            case 63: ModConfig.showSaturation = !ModConfig.showSaturation; break;
            case 64:
                if (ModConfig.itemTracker2d || ModConfig.itemTracker3d) {
                    ModConfig.itemTracker2d = false;
                    ModConfig.itemTracker3d = false;
                } else {
                    ModConfig.itemTracker2d = true;
                    ModConfig.itemTracker3d = true;
                }
                break;
            case 65: ModConfig.showWatermark = !ModConfig.showWatermark; break;
            case 66: ModConfig.motionBlurEnabled = !ModConfig.motionBlurEnabled; break;
            case 67: ModConfig.chatUnlimited = !ModConfig.chatUnlimited; break;
            case 68: ModConfig.tabEditor = !ModConfig.tabEditor; break;
            case 70: ModConfig.hitColorEnabled = !ModConfig.hitColorEnabled; break;
            case 71: ModConfig.showPackHud = !ModConfig.showPackHud; break;
            case 72:
                if (ModConfig.hotbarScale != 100 || ModConfig.inventoryScale != 100) {
                    ModConfig.hotbarScale = 100;
                    ModConfig.inventoryScale = 100;
                } else {
                    ModConfig.hotbarScale = 125;
                }
                break;
            case 73: ModConfig.showHeightLimit = !ModConfig.showHeightLimit; break;
            case 74:
                ModConfig.cycleCrosshairMode();
                break;
            case 81:
                ModConfig.autoGameModeProfiles = !ModConfig.autoGameModeProfiles;
                ModConfig.autoBedwarsHud = ModConfig.autoGameModeProfiles;
                break;
            case 82:
                GameModeDetector.cycleOverride();
                break;
            default: break;
        }
    }

    @Override
    public boolean doesGuiPauseGame() {
        return false;
    }
}
