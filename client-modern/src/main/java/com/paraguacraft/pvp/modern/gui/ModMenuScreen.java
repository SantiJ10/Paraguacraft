package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.config.LauncherProfile;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.FullbrightManager;
import com.paraguacraft.pvp.modern.core.GameModeDetector;
import com.paraguacraft.pvp.modern.core.PerformanceBootstrap;
import com.paraguacraft.pvp.modern.core.PerformanceConfig;
import com.paraguacraft.pvp.modern.core.PlayStyle;
import com.paraguacraft.pvp.modern.core.ServerContext;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.Click;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.TextFieldWidget;
import net.minecraft.text.Text;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Locale;
import java.util.Set;
import java.util.function.BooleanSupplier;
import java.util.function.Consumer;

/** Hub Right Shift: overlay compacto, pestañas Mods / Ajustes / Packs. */
public class ModMenuScreen extends ParaguacraftScreen {

    private static final String[] TABS = {"Mods", "Ajustes", "Packs"};
    private static final int TOPBAR = 72;
    private static final int FOOTER = 30;
    private static final int TAB_MODS = 0;
    private static final int TAB_SETTINGS = 1;
    private static final int TAB_PACKS = 2;

    private int selectedTab;
    private String search = "";
    private float scroll;
    private TextFieldWidget searchField;

    public ModMenuScreen(Screen parent) {
        super(Text.literal("Paraguacraft Mods"), parent);
    }

    private int[] panelGeom() {
        int panelW = Math.min(width - 32, 720);
        int panelH = Math.min(height - 28, 400);
        int panelX = (width - panelW) / 2;
        int panelY = Math.max(8, (height - panelH) / 2);
        return new int[] {panelX, panelY, panelW, panelH};
    }

    @Override
    protected void init() {
        int[] g = panelGeom();
        int panelX = g[0];
        int panelY = g[1];
        int panelW = g[2];
        int panelH = g[3];
        int gridX = panelX + 10;
        int gridY = panelY + TOPBAR + 8;
        int gridW = panelW - 20;
        int cardW = Math.min(168, (gridW - 12) / 2);
        int cardH = 56;
        int gap = 8;

        List<ModCard> cards = filteredCards();
        int rec = recommendedCount(cards);
        int headerH = rec > 0 ? 14 : 0;
        int cols = Math.max(1, gridW / (cardW + gap));
        int row = 0;
        int col = 0;
        for (ModCard card : cards) {
            int x = gridX + col * (cardW + gap);
            int y = gridY + headerH + row * (cardH + gap) - (int) scroll;
            if (y + cardH >= panelY + TOPBAR && y <= panelY + panelH - FOOTER) {
                addCardButton(card, x, y, cardW, cardH);
            }
            col++;
            if (col >= cols) {
                col = 0;
                row++;
            }
        }

        int closeY = panelY + panelH - 24;
        addDrawableChild(FlatMenuButton.create(panelX + 10, closeY, 100, 18,
            Text.literal("Editar HUD"), () -> client.setScreen(new GuiEditHudScreen(this))));
        addDrawableChild(FlatMenuButton.create(panelX + 114, closeY, 90, 18,
            Text.literal("Atajos"), () -> client.setScreen(
                new net.minecraft.client.gui.screen.option.ControlsOptionsScreen(this, client.options))));
        addDrawableChild(FlatMenuButton.create(panelX + 208, closeY, 110, 18,
            Text.literal("Exportar JSON"), () -> client.setScreen(new ModProfilesScreen(this))));
        addDrawableChild(FlatMenuButton.create(panelX + panelW - 100, closeY, 90, 18,
            Text.literal("Cerrar"), this::goBack));

        if (selectedTab == TAB_MODS) {
            initSearchField(panelX, panelY, panelW);
        } else {
            searchField = null;
        }
    }

    private void initSearchField(int panelX, int panelY, int panelW) {
        int fieldW = 160;
        int fieldX = panelX + panelW - fieldW - 14;
        int fieldY = panelY + 48;
        if (searchField == null) {
            searchField = new TextFieldWidget(textRenderer, fieldX, fieldY, fieldW, 16, Text.literal("Buscar"));
            searchField.setMaxLength(32);
            searchField.setPlaceholder(Text.literal("Buscar mods…"));
            searchField.setText(search);
            searchField.setChangedListener(text -> {
                search = text;
                scroll = 0;
                clearChildren();
                init();
            });
        } else {
            searchField.setX(fieldX);
            searchField.setY(fieldY);
        }
        addDrawableChild(searchField);
        setFocused(searchField);
    }

    private void addCardButton(ModCard card, int x, int y, int w, int h) {
        if ("open".equals(card.action)) {
            addDrawableChild(MenuCardButton.open(x, y, w, h, card.label, () -> open(card.openTarget)));
        } else {
            addDrawableChild(MenuCardButton.create(x, y, w, h, card.label, card.getter.getAsBoolean(), () -> {
                card.setter.accept(!card.getter.getAsBoolean());
                if ("Fullbright".equals(card.label) && !ModernConfig.fullbright) {
                    client.options.getGamma().setValue(1.0);
                }
                ModernConfig.save();
                if ("Pantalla sin bordes".equals(card.label)) {
                    com.paraguacraft.pvp.modern.core.WindowedFullscreenManager.sync(client);
                }
                clearChildren();
                init();
            }));
        }
    }

    private void open(String target) {
        switch (target) {
            case "quickplay" -> client.setScreen(new QuickPlayChooserScreen(this));
            case "cubecraft_qp" -> client.setScreen(new CubecraftQuickPlayScreen(this));
            case "chat_alerts" -> client.setScreen(new GuiChatAlertsScreen(this));
            case "gamemode_override" -> client.setScreen(new GuiGameModeOverrideScreen(this));
            case "fullbright", "gamma_utils" -> {
                FullbrightManager.toggle(client);
                clearChildren();
                init();
            }
            case "chat_triggers_cfg" -> client.setScreen(new GuiChatTriggersScreen(this));
            case "packs" -> client.setScreen(new PackSelectScreen(this));
            case "theme" -> client.setScreen(new ThemeSelectScreen(this));
            case "nickfinder" -> client.setScreen(new NickFinderScreen(this));
            case "music_hud" -> client.setScreen(new GuiMusicHudOptionsScreen(this));
            case "motion_blur" -> client.setScreen(new GuiMotionBlurOptionsScreen(this));
            case "armor_hud" -> client.setScreen(GuiSubmodOptionsScreen.armor(this));
            case "keystrokes" -> client.setScreen(GuiSubmodOptionsScreen.keystrokes(this));
            case "cosmetics" -> client.setScreen(GuiSubmodOptionsScreen.cosmetics(this));
            case "pvp_trackers" -> client.setScreen(GuiSubmodOptionsScreen.pvpTrackers(this));
            case "fps_group" -> client.setScreen(GuiSubmodOptionsScreen.fps(this));
            case "entity_group" -> client.setScreen(GuiSubmodOptionsScreen.entity(this));
            case "bedwars_group" -> client.setScreen(GuiSubmodOptionsScreen.bedwars(this));
            case "chat_group" -> client.setScreen(GuiSubmodOptionsScreen.chat(this));
            case "scoreboard_group" -> client.setScreen(GuiSubmodOptionsScreen.scoreboard(this));
            case "tnt_group" -> client.setScreen(GuiSubmodOptionsScreen.tnt(this));
            case "tab_group" -> client.setScreen(GuiSubmodOptionsScreen.tab(this));
            case "hit_color" -> client.setScreen(GuiSubmodOptionsScreen.hitColor(this));
            case "particles" -> client.setScreen(GuiSubmodOptionsScreen.particles(this));
            case "reach_group" -> client.setScreen(GuiSubmodOptionsScreen.reach(this));
            case "gui_scale" -> client.setScreen(GuiSubmodOptionsScreen.guiScale(this));
            case "pack_hud" -> client.setScreen(GuiSubmodOptionsScreen.packHud(this));
            case "height_limit" -> client.setScreen(GuiSubmodOptionsScreen.heightLimit(this));
            case "badges_ping" -> client.setScreen(new GuiBadgesPingOptionsScreen(this));
            case "sprint" -> client.setScreen(new GuiSprintOptionsScreen(this));
            case "crosshair" -> {
                ModernConfig.cycleCrosshairMode();
                ModernConfig.save();
                clearChildren();
                init();
            }
            case "profile_cycle" -> {
                GameModeDetector.cycleOverride();
                clearChildren();
                init();
            }
            case "clean_memory" -> PerformanceBootstrap.cleanMemoryNow();
            case "apply_hw_preset" -> PerformanceBootstrap.applyPresetNow(client);
            default -> {}
        }
    }

    private List<ModCard> filteredCards() {
        List<ModCard> all = allCards();
        List<ModCard> tabbed = new ArrayList<>();
        String q = search.toLowerCase(Locale.ROOT).trim();
        for (ModCard card : all) {
            if (selectedTab == TAB_SETTINGS && card.tab != TAB_SETTINGS) {
                continue;
            }
            if (selectedTab == TAB_PACKS && card.tab != TAB_PACKS) {
                continue;
            }
            if (selectedTab == TAB_MODS && card.tab != TAB_MODS) {
                continue;
            }
            if (selectedTab == TAB_MODS && !q.isEmpty() && !card.label.toLowerCase(Locale.ROOT).contains(q)) {
                continue;
            }
            tabbed.add(card);
        }
        if (selectedTab != TAB_MODS || !q.isEmpty()) {
            return tabbed;
        }
        String[] rec = recommendedKeys();
        List<ModCard> out = new ArrayList<>();
        Set<String> seen = new HashSet<>();
        for (String key : rec) {
            for (ModCard card : tabbed) {
                if (key.equals(card.recKey) && seen.add(key)) {
                    out.add(card);
                    break;
                }
            }
        }
        for (ModCard card : tabbed) {
            if (card.recKey == null || card.recKey.isEmpty() || !seen.contains(card.recKey)) {
                out.add(card);
            } else if (seen.contains(card.recKey) && !out.contains(card)) {
                // already added as recommended
            }
        }
        // Dedup: skip cards already in out
        List<ModCard> unique = new ArrayList<>();
        Set<ModCard> used = new HashSet<>(out);
        unique.addAll(out);
        for (ModCard card : tabbed) {
            if (!used.contains(card)) {
                unique.add(card);
            }
        }
        return unique;
    }

    private int recommendedCount(List<ModCard> cards) {
        if (selectedTab != TAB_MODS || !search.isBlank()) {
            return 0;
        }
        String[] rec = recommendedKeys();
        int n = 0;
        for (String key : rec) {
            for (ModCard card : cards) {
                if (key.equals(card.recKey)) {
                    n++;
                    break;
                }
            }
        }
        return n;
    }

    private static String[] recommendedKeys() {
        return switch (GameModeDetector.current()) {
            case BEDWARS -> new String[] {"bedwars", "height", "tnt", "armor", "compass"};
            case SKYWARS, LUCKY_ISLANDS -> new String[] {"armor", "potions", "held", "reach"};
            case DUELS -> new String[] {"reach", "combo", "hit_color", "armor", "badges"};
            case UHC -> new String[] {"coords", "armor", "potions", "compass"};
            case LOBBY -> new String[] {"server", "packs", "quickplay"};
            default -> new String[] {"reach", "combo", "armor", "nohurt"};
        };
    }

    private List<ModCard> allCards() {
        List<ModCard> cards = new ArrayList<>();
        cards.add(open(TAB_MODS, "FPS", "fps_group", ""));
        cards.add(toggle(TAB_MODS, "Ping", () -> ModernConfig.showPing, v -> ModernConfig.showPing = v, ""));
        cards.add(toggle(TAB_MODS, "CPS", () -> ModernConfig.showCps, v -> ModernConfig.showCps = v, ""));
        cards.add(open(TAB_MODS, "Keystrokes", "keystrokes", ""));
        cards.add(toggle(TAB_MODS, "Coordenadas", () -> ModernConfig.showCoords, v -> ModernConfig.showCoords = v, "coords"));
        cards.add(open(TAB_MODS, "Armadura HUD", "armor_hud", "armor"));
        cards.add(open(TAB_MODS, "Cosmeticos / nametags", "cosmetics", ""));
        cards.add(open(TAB_MODS, "Waypoints y trackers", "pvp_trackers", ""));
        cards.add(toggle(TAB_MODS, "Contador bloques", () -> ModernConfig.showBlockCount, v -> ModernConfig.showBlockCount = v, ""));
        cards.add(toggle(TAB_MODS, "Objeto en mano", () -> ModernConfig.showHeldItem, v -> ModernConfig.showHeldItem = v, "held"));
        cards.add(open(TAB_MODS, "BedWars", "bedwars_group", "bedwars"));
        cards.add(open(TAB_MODS, "Limite de altura", "height_limit", "height"));
        cards.add(toggle(TAB_MODS, "Hardware HUD", () -> ModernConfig.showHardwareHud, v -> ModernConfig.showHardwareHud = v, ""));
        cards.add(open(TAB_MODS, "Musica", "music_hud", ""));
        cards.add(toggle(TAB_MODS, "Pociones HUD", () -> ModernConfig.showPotions, v -> ModernConfig.showPotions = v, "potions"));
        cards.add(toggle(TAB_MODS, "Brújula", () -> ModernConfig.showCompass, v -> ModernConfig.showCompass = v, "compass"));
        cards.add(toggle(TAB_MODS, "Combo counter", () -> ModernConfig.comboCounter, v -> ModernConfig.comboCounter = v, "combo"));
        cards.add(toggle(TAB_MODS, "Hitbox azul", () -> ModernConfig.showBlockOutline, v -> ModernConfig.showBlockOutline = v, ""));
        cards.add(toggle(TAB_MODS, "No hurt cam", () -> ModernConfig.noHurtCam, v -> ModernConfig.noHurtCam = v, "nohurt"));
        cards.add(toggle(TAB_MODS, "Low fire", () -> ModernConfig.lowFire, v -> ModernConfig.lowFire = v, ""));
        cards.add(toggle(TAB_MODS, "Item physics", () -> ModernConfig.itemPhysics, v -> ModernConfig.itemPhysics = v, ""));
        cards.add(open(TAB_MODS, "TNT countdown", "tnt_group", "tnt"));
        cards.add(open(TAB_MODS, "Reach display", "reach_group", "reach"));
        cards.add(open(TAB_MODS, "Color de golpe", "hit_color", "hit_color"));
        cards.add(open(TAB_MODS, "Editor de Tab", "tab_group", ""));
        cards.add(toggle(TAB_MODS, "Estadisticas de combate", () -> ModernConfig.showCombatStatsHud, v -> ModernConfig.showCombatStatsHud = v, ""));
        cards.add(toggle(TAB_MODS, "HUD servidor (nombre/IP)", () -> ModernConfig.showServerHud, v -> ModernConfig.showServerHud = v, "server"));
        cards.add(open(TAB_MODS, "Sprint", "sprint", ""));
        cards.add(toggle(TAB_MODS, "Toggle sneak (Shift)", () -> ModernConfig.toggleSneak, v -> {
            ModernConfig.toggleSneak = v;
            ModernConfig.isSneakingToggled = false;
        }, ""));
        cards.add(open(TAB_MODS, FullbrightManager.menuLabel(), "fullbright", ""));
        cards.add(toggle(TAB_MODS, "Freelook (Alt)", () -> ModernConfig.freelookEnabled, v -> ModernConfig.freelookEnabled = v, ""));
        cards.add(toggle(TAB_MODS, "Freelook blacklist ranked", () -> ModernConfig.freelookBlacklistServers, v -> ModernConfig.freelookBlacklistServers = v, ""));
        cards.add(toggle(TAB_MODS, "Shaders auto-off en partida", () -> ModernConfig.shaderAutoOffInMatch, v -> ModernConfig.shaderAutoOffInMatch = v, ""));
        cards.add(toggle(TAB_MODS, "Reach solo practica", () -> ModernConfig.reachDisplayPracticeOnly, v -> ModernConfig.reachDisplayPracticeOnly = v, ""));
        cards.add(toggle(TAB_MODS, "Animaciones 1.7 (swing/blockhit espada)", () -> ModernConfig.oldAnimations, v -> {
            ModernConfig.oldAnimations = v;
            PerformanceConfig.oldAnimations = v;
        }, ""));
        cards.add(toggle(TAB_MODS, "Ocultar titulos", () -> ModernConfig.hideTitles, v -> ModernConfig.hideTitles = v, ""));
        cards.add(open(TAB_MODS, "Chat", "chat_group", ""));
        cards.add(open(TAB_MODS, "Config chat triggers", "chat_triggers_cfg", ""));
        cards.add(open(TAB_MODS, "Config chat alerts", "chat_alerts", ""));
        cards.add(open(TAB_MODS, "Scoreboard", "scoreboard_group", ""));
        cards.add(toggle(TAB_MODS, "HUD modo de juego", () -> ModernConfig.showGameModeHud, v -> ModernConfig.showGameModeHud = v, ""));
        cards.add(toggle(TAB_MODS, "Boost FPS", () -> PerformanceConfig.boostFps, v -> {
            PerformanceConfig.boostFps = v;
            ModernConfig.save();
        }, ""));
        cards.add(open(TAB_MODS, "Entity / cull", "entity_group", ""));
        cards.add(open(TAB_MODS, "Particulas", "particles", ""));
        cards.add(open(TAB_MODS, "Limpiar memoria", "clean_memory", ""));
        cards.add(open(TAB_MODS, "Aplicar preset de hardware", "apply_hw_preset", ""));
        cards.add(toggle(TAB_MODS, "Full rendimiento (vs Casual)", () -> PlayStyle.isCompetitive(), v -> {
            LauncherProfile.playStyle = v ? "competitive" : "casual";
            PerformanceBootstrap.applyPresetNow(MinecraftClient.getInstance());
            ModernConfig.save();
        }, ""));
        cards.add(open(TAB_MODS, "Quick Play (`)", "quickplay", "quickplay"));
        cards.add(open(TAB_MODS, "Cubecraft Quick Play", "cubecraft_qp", ""));
        cards.add(open(TAB_MODS, "NickFinder (N)", "nickfinder", ""));
        cards.add(toggle(TAB_MODS, "Colores de equipo", () -> ModernConfig.teamColors, v -> ModernConfig.teamColors = v, ""));
        cards.add(toggle(TAB_MODS, "NickFinder activo", () -> ModernConfig.nickFinderEnabled, v -> ModernConfig.nickFinderEnabled = v, ""));
        cards.add(open(TAB_MODS, "Insignias y Ping", "badges_ping", "badges"));
        cards.add(open(TAB_MODS, "Mira: " + ModernConfig.crosshairModeLabel(), "crosshair", ""));
        cards.add(toggle(TAB_MODS, "Saturacion", () -> ModernConfig.showSaturation, v -> ModernConfig.showSaturation = v, ""));

        cards.add(open(TAB_SETTINGS, "Mira: " + ModernConfig.crosshairModeLabel(), "crosshair", ""));
        cards.add(toggle(TAB_SETTINGS, "Perfiles auto por modo", () -> ModernConfig.autoGameModeProfiles, v -> ModernConfig.autoGameModeProfiles = v, ""));
        cards.add(open(TAB_SETTINGS, "Perfil: " + GameModeDetector.overrideLabel(), "profile_cycle", ""));
        cards.add(open(TAB_SETTINGS, "Modo de juego (manual)", "gamemode_override", ""));
        cards.add(toggle(TAB_SETTINGS, "Pantalla sin bordes", () -> ModernConfig.windowedFullscreen, v -> ModernConfig.windowedFullscreen = v, ""));
        cards.add(open(TAB_SETTINGS, "Motion Blur", "motion_blur", ""));
        cards.add(open(TAB_SETTINGS, "Escala GUI", "gui_scale", ""));
        cards.add(toggle(TAB_SETTINGS, "FOV estatico", () -> !ModernConfig.dynamicFov, v -> {
            ModernConfig.dynamicFov = !v;
            if (client != null && client.options != null && client.options.getFovEffectScale() != null) {
                client.options.getFovEffectScale().setValue(ModernConfig.dynamicFov ? 1.0 : 0.0);
            }
        }, ""));
        cards.add(toggle(TAB_SETTINGS, "Saturacion", () -> ModernConfig.showSaturation, v -> ModernConfig.showSaturation = v, ""));
        cards.add(toggle(TAB_SETTINGS, "UI " + ModernConfig.uiScaleLabel(), () -> true, v -> {
            ModernConfig.cycleUiScale();
            ModernConfig.save();
        }, ""));

        cards.add(open(TAB_PACKS, "Texture packs", "packs", "packs"));
        cards.add(open(TAB_PACKS, "Visualizacion de pack", "pack_hud", ""));
        cards.add(open(TAB_PACKS, "Tema del menu", "theme", ""));
        return cards;
    }

    private static ModCard toggle(int tab, String label, BooleanSupplier getter, Consumer<Boolean> setter, String recKey) {
        ModCard c = new ModCard();
        c.tab = tab;
        c.label = label;
        c.getter = getter;
        c.setter = setter;
        c.action = "toggle";
        c.recKey = recKey;
        return c;
    }

    private static ModCard open(int tab, String label, String target, String recKey) {
        ModCard c = new ModCard();
        c.tab = tab;
        c.label = label;
        c.action = "open";
        c.openTarget = target;
        c.recKey = recKey;
        c.getter = () -> false;
        c.setter = v -> {};
        return c;
    }

    @Override
    public boolean mouseScrolled(double mouseX, double mouseY, double horizontal, double vertical) {
        scroll = Math.max(0, scroll - (float) vertical * 16f);
        clearChildren();
        init();
        return true;
    }

    @Override
    public boolean mouseClicked(Click click, boolean doubled) {
        int[] g = panelGeom();
        int panelX = g[0];
        int panelY = g[1];
        int tabY = panelY + 24;
        int tabW = 78;
        for (int i = 0; i < TABS.length; i++) {
            int tx = panelX + 12 + i * (tabW + 6);
            if (click.x() >= tx && click.x() < tx + tabW && click.y() >= tabY && click.y() < tabY + 18) {
                selectedTab = i;
                scroll = 0;
                searchField = null;
                clearChildren();
                init();
                return true;
            }
        }
        return super.mouseClicked(click, doubled);
    }

    @Override
    public void renderBackground(DrawContext ctx, int mouseX, int mouseY, float delta) {
        if (client != null && client.world != null) {
            ctx.fill(0, 0, width, height, 0x44000000);
        } else {
            super.renderBackground(ctx, mouseX, mouseY, delta);
        }
        int[] g = panelGeom();
        int panelX = g[0];
        int panelY = g[1];
        int panelW = g[2];
        int panelH = g[3];
        ctx.fill(panelX, panelY, panelX + panelW, panelY + panelH, 0xCC0A0C14);
        ctx.fill(panelX, panelY, panelX + panelW, panelY + 1, 0x33FFFFFF);
        ctx.fill(panelX, panelY + panelH - FOOTER, panelX + panelW, panelY + panelH, 0xAA080A10);
    }

    @Override
    public void render(DrawContext ctx, int mouseX, int mouseY, float delta) {
        super.render(ctx, mouseX, mouseY, delta);
        int[] g = panelGeom();
        int panelX = g[0];
        int panelY = g[1];
        int panelW = g[2];
        ctx.drawText(textRenderer, Text.literal("PARAGUACRAFT"), panelX + 12, panelY + 8, UiTheme.accent(), true);
        String chip = ServerContext.chip(client);
        int chipW = textRenderer.getWidth(chip) + 12;
        int chipX = panelX + panelW - chipW - 12;
        ctx.fill(chipX, panelY + 6, chipX + chipW, panelY + 20, 0xAA123040);
        ctx.drawText(textRenderer, Text.literal(chip), chipX + 6, panelY + 9, UiTheme.accent(), true);

        int tabY = panelY + 24;
        int tabW = 78;
        for (int i = 0; i < TABS.length; i++) {
            int tx = panelX + 12 + i * (tabW + 6);
            if (i == selectedTab) {
                ctx.fill(tx, tabY, tx + tabW, tabY + 18, 0x4400E5FF);
            }
            int color = i == selectedTab ? UiTheme.TEXT : UiTheme.textDim();
            int tw = textRenderer.getWidth(TABS[i]);
            ctx.drawText(textRenderer, Text.literal(TABS[i]), tx + tabW / 2 - tw / 2, tabY + 5, color, true);
        }

        if (selectedTab == TAB_MODS && search.isBlank() && recommendedCount(filteredCards()) > 0) {
            ctx.drawText(textRenderer, Text.literal("Para este modo"), panelX + 12, panelY + TOPBAR - 2, UiTheme.accent(), true);
        }
    }

    private void goBack() {
        ModernConfig.save();
        client.setScreen(parent != null ? parent : new CustomTitleScreen());
    }

    private static final class ModCard {
        int tab;
        String label;
        String action;
        String openTarget;
        String recKey;
        BooleanSupplier getter;
        Consumer<Boolean> setter;
    }
}
