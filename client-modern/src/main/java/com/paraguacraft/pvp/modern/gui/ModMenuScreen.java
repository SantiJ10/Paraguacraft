package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.PerformanceBootstrap;
import com.paraguacraft.pvp.modern.core.PerformanceConfig;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.Click;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.TextFieldWidget;
import net.minecraft.text.Text;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.function.BooleanSupplier;
import java.util.function.Consumer;

/** Mod Menu estilo Lunar / Paraguacraft 1.8.9 con categorias y tarjetas. */
public class ModMenuScreen extends ParaguacraftScreen {

    private static final String[] CATEGORIES = {"Todos", "HUD", "PvP", "Mecanicas", "Rendimiento", "Hypixel"};
    private static final int SIDEBAR = 132;
    private static final int TOPBAR = 44;

    private int selectedCategory;
    private String search = "";
    private float scroll;
    private TextFieldWidget searchField;

    public ModMenuScreen(Screen parent) {
        super(Text.literal("Paraguacraft Mods"), parent);
    }

    @Override
    protected void init() {
        int panelX = Math.max(8, width / 2 - 420);
        int panelY = 20;
        int panelW = Math.min(width - 16, 840);
        int panelH = height - 40;
        int gridX = panelX + SIDEBAR + 8;
        int gridY = panelY + TOPBAR + 8;
        int gridW = panelW - SIDEBAR - 16;
        int cardW = Math.min(168, (gridW - 12) / 2);
        int cardH = 56;
        int gap = 8;

        List<ModCard> cards = filteredCards();
        int cols = Math.max(1, gridW / (cardW + gap));
        int row = 0;
        int col = 0;
        for (ModCard card : cards) {
            int x = gridX + col * (cardW + gap);
            int y = gridY + row * (cardH + gap) - (int) scroll;
            if (y + cardH >= panelY + TOPBAR && y <= panelY + panelH - 8) {
                addCardButton(card, x, y, cardW, cardH);
            }
            col++;
            if (col >= cols) {
                col = 0;
                row++;
            }
        }

        int closeY = panelY + panelH - 26;
        addDrawableChild(FlatMenuButton.create(panelX + 12, closeY, 100, 20,
            Text.literal("Editar HUD"), () -> client.setScreen(new GuiEditHudScreen(this))));
        addDrawableChild(FlatMenuButton.create(panelX + 118, closeY, 100, 20,
            Text.literal("Atajos"), () -> client.setScreen(
                new net.minecraft.client.gui.screen.option.ControlsOptionsScreen(this, client.options))));
        addDrawableChild(FlatMenuButton.create(panelX + 224, closeY, 100, 20,
            Text.literal("Perfiles"), () -> client.setScreen(new ModProfilesScreen(this))));
        addDrawableChild(FlatMenuButton.create(panelX + panelW - 112, closeY, 100, 20,
            Text.literal("Cerrar"), this::goBack));

        initSearchField(panelX, panelY, panelW);
    }

    /** Campo de busqueda real (teclado nativo: backspace, pegar, seleccion). */
    private void initSearchField(int panelX, int panelY, int panelW) {
        int fieldW = 160;
        int fieldX = panelX + panelW - fieldW - 14;
        int fieldY = panelY + 12;
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
            case "quickplay" -> client.setScreen(new HypixelQuickPlayScreen(this));
            case "packs" -> client.setScreen(new PackSelectScreen(this));
            case "theme" -> client.setScreen(new ThemeSelectScreen(this));
            case "nickfinder" -> client.setScreen(new NickFinderScreen(this));
            case "music_hud" -> client.setScreen(new GuiMusicHudOptionsScreen(this));
            case "badges_ping" -> client.setScreen(new GuiBadgesPingOptionsScreen(this));
            case "sprint" -> client.setScreen(new GuiSprintOptionsScreen(this));
            case "crosshair" -> {
                ModernConfig.cycleCrosshairMode();
                ModernConfig.save();
                clearChildren();
                init();
            }
            case "particles" -> {
                PerformanceConfig.cycleParticleMode();
                PerformanceBootstrap.applyParticleModeNow(client);
                ModernConfig.save();
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
        List<ModCard> out = new ArrayList<>();
        String q = search.toLowerCase(Locale.ROOT).trim();
        for (ModCard card : all) {
            if (selectedCategory > 0 && card.category != selectedCategory) {
                continue;
            }
            if (!q.isEmpty() && !card.label.toLowerCase(Locale.ROOT).contains(q)) {
                continue;
            }
            out.add(card);
        }
        return out;
    }

    private List<ModCard> allCards() {
        List<ModCard> cards = new ArrayList<>();
        cards.add(toggle(1, "FPS", () -> ModernConfig.showFps, v -> ModernConfig.showFps = v));
        cards.add(toggle(1, "Ping", () -> ModernConfig.showPing, v -> ModernConfig.showPing = v));
        cards.add(toggle(1, "CPS", () -> ModernConfig.showCps, v -> ModernConfig.showCps = v));
        cards.add(toggle(1, "Keystrokes", () -> ModernConfig.showKeystrokes, v -> ModernConfig.showKeystrokes = v));
        cards.add(toggle(1, "Coordenadas", () -> ModernConfig.showCoords, v -> ModernConfig.showCoords = v));
        cards.add(toggle(1, "Armadura + iconos", () -> ModernConfig.showArmor, v -> ModernConfig.showArmor = v));
        cards.add(toggle(1, "Contador bloques", () -> ModernConfig.showBlockCount, v -> ModernConfig.showBlockCount = v));
        cards.add(toggle(1, "Objeto en mano", () -> ModernConfig.showHeldItem, v -> ModernConfig.showHeldItem = v));
        cards.add(toggle(1, "Recursos BedWars", () -> ModernConfig.showBedwarsResources, v -> ModernConfig.showBedwarsResources = v));
        cards.add(toggle(1, "BW fondo transparente", () -> ModernConfig.bwResTransparentBg, v -> ModernConfig.bwResTransparentBg = v));
        cards.add(toggle(1, "Hardware HUD", () -> ModernConfig.showHardwareHud, v -> ModernConfig.showHardwareHud = v));
        cards.add(open(1, "Musica", "music_hud"));
        cards.add(toggle(1, "Pociones HUD", () -> ModernConfig.showPotions, v -> ModernConfig.showPotions = v));
        cards.add(toggle(1, "Brújula", () -> ModernConfig.showCompass, v -> ModernConfig.showCompass = v));
        cards.add(toggle(1, "Combo counter", () -> ModernConfig.comboCounter, v -> ModernConfig.comboCounter = v));
        cards.add(toggle(2, "Hitbox azul", () -> ModernConfig.showBlockOutline, v -> ModernConfig.showBlockOutline = v));
        cards.add(toggle(2, "No hurt cam", () -> ModernConfig.noHurtCam, v -> ModernConfig.noHurtCam = v));
        cards.add(toggle(2, "Low fire", () -> ModernConfig.lowFire, v -> ModernConfig.lowFire = v));
        cards.add(toggle(2, "Item physics", () -> ModernConfig.itemPhysics, v -> ModernConfig.itemPhysics = v));
        cards.add(toggle(2, "TNT countdown", () -> ModernConfig.showTntCountdown, v -> ModernConfig.showTntCountdown = v));
        cards.add(toggle(2, "Reach display", () -> ModernConfig.reachDisplay, v -> ModernConfig.reachDisplay = v));
        cards.add(open(2, "Insignias y Ping", "badges_ping"));
        cards.add(toggle(2, "Estadisticas de combate", () -> ModernConfig.showCombatStatsHud, v -> ModernConfig.showCombatStatsHud = v));
        cards.add(toggle(1, "HUD servidor (nombre/IP)", () -> ModernConfig.showServerHud, v -> ModernConfig.showServerHud = v));
        cards.add(open(3, "Sprint", "sprint"));
        cards.add(toggle(3, "Toggle sneak (Shift)", () -> ModernConfig.toggleSneak, v -> {
            ModernConfig.toggleSneak = v;
            ModernConfig.isSneakingToggled = false;
        }));
        cards.add(toggle(3, "Pantalla sin bordes", () -> ModernConfig.windowedFullscreen, v -> ModernConfig.windowedFullscreen = v));
        cards.add(toggle(3, "Fullbright", () -> ModernConfig.fullbright, v -> ModernConfig.fullbright = v));
        cards.add(toggle(3, "FOV dinamico", () -> ModernConfig.dynamicFov, v -> ModernConfig.dynamicFov = v));
        cards.add(toggle(3, "Freelook (Alt)", () -> ModernConfig.freelookEnabled, v -> ModernConfig.freelookEnabled = v));
        cards.add(toggle(3, "Old animations", () -> ModernConfig.oldAnimations, v -> ModernConfig.oldAnimations = v));
        cards.add(toggle(3, "Ocultar titulos", () -> ModernConfig.hideTitles, v -> ModernConfig.hideTitles = v));
        cards.add(toggle(3, "Chat triggers", () -> ModernConfig.chatTriggers, v -> ModernConfig.chatTriggers = v));
        cards.add(toggle(3, "Scoreboard", () -> ModernConfig.scoreboardEnabled, v -> ModernConfig.scoreboardEnabled = v));
        cards.add(toggle(4, "Boost FPS", () -> PerformanceConfig.boostFps, v -> {
            PerformanceConfig.boostFps = v;
            ModernConfig.save();
        }));
        cards.add(toggle(4, "Entity cull", () -> ModernConfig.entityCull, v -> ModernConfig.entityCull = v));
        cards.add(toggle(4, "Nametag cull", () -> ModernConfig.nametagCull, v -> ModernConfig.nametagCull = v));
        cards.add(toggle(4, "FPS bajo sin foco", () -> PerformanceConfig.reduceFpsWhenMinimized, v -> {
            PerformanceConfig.reduceFpsWhenMinimized = v;
            ModernConfig.save();
        }));
        cards.add(toggle(4, "Armadura %", () -> ModernConfig.showArmorPercentage, v -> ModernConfig.showArmorPercentage = v));
        cards.add(open(4, "Particulas: " + PerformanceConfig.particleModeLabel(), "particles"));
        cards.add(open(4, "Limpiar memoria", "clean_memory"));
        cards.add(open(4, "Aplicar preset de hardware", "apply_hw_preset"));
        cards.add(open(4, "Crosshair: " + ModernConfig.crosshairModeLabel(), "crosshair"));
        cards.add(open(5, "Hypixel Quick Play", "quickplay"));
        cards.add(open(5, "Texture packs", "packs"));
        cards.add(open(5, "NickFinder (N)", "nickfinder"));
        cards.add(toggle(5, "Camas coloridas BW", () -> ModernConfig.coloredBeds, v -> ModernConfig.coloredBeds = v));
        cards.add(toggle(5, "Colores de equipo", () -> ModernConfig.teamColors, v -> ModernConfig.teamColors = v));
        cards.add(toggle(5, "NickFinder activo", () -> ModernConfig.nickFinderEnabled, v -> ModernConfig.nickFinderEnabled = v));
        cards.add(open(5, "Tema del menu", "theme"));
        return cards;
    }

    private static ModCard toggle(int category, String label, BooleanSupplier getter, Consumer<Boolean> setter) {
        ModCard c = new ModCard();
        c.category = category;
        c.label = label;
        c.getter = getter;
        c.setter = setter;
        c.action = "toggle";
        return c;
    }

    private static ModCard open(int category, String label, String target) {
        ModCard c = new ModCard();
        c.category = category;
        c.label = label;
        c.action = "open";
        c.openTarget = target;
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
        int panelX = Math.max(8, width / 2 - 420);
        int panelY = 20;
        int panelH = height - 40;
        if (click.x() >= panelX && click.x() < panelX + SIDEBAR && click.y() >= panelY + TOPBAR && click.y() < panelY + panelH - 8) {
            int idx = (int) ((click.y() - panelY - TOPBAR) / 22);
            if (idx >= 0 && idx < CATEGORIES.length) {
                selectedCategory = idx;
                scroll = 0;
                clearChildren();
                init();
                return true;
            }
        }
        return super.mouseClicked(click, doubled);
    }

    @Override
    public void renderBackground(DrawContext ctx, int mouseX, int mouseY, float delta) {
        super.renderBackground(ctx, mouseX, mouseY, delta);
        int panelX = Math.max(8, width / 2 - 420);
        int panelY = 20;
        int panelW = Math.min(width - 16, 840);
        int panelH = height - 40;
        ctx.fill(panelX, panelY, panelX + panelW, panelY + panelH, 0xCC0A0C14);
        ctx.fill(panelX, panelY, panelX + panelW, panelY + 1, 0x33FFFFFF);
        ctx.fill(panelX, panelY + TOPBAR, panelX + SIDEBAR, panelY + panelH, 0xBB080A10);
    }

    @Override
    public void render(DrawContext ctx, int mouseX, int mouseY, float delta) {
        super.render(ctx, mouseX, mouseY, delta);
        int panelX = Math.max(8, width / 2 - 420);
        int panelY = 20;
        ctx.drawText(textRenderer, Text.literal("PARAGUACRAFT"), panelX + 14, panelY + 12, UiTheme.accent(), true);
        ctx.drawText(textRenderer, Text.literal("Mod Menu"), panelX + 14, panelY + 26, UiTheme.textDim(), true);
        int catY = panelY + TOPBAR + 8;
        for (int i = 0; i < CATEGORIES.length; i++) {
            if (i == selectedCategory) {
                ctx.fill(panelX + 6, catY + i * 22 - 2, panelX + SIDEBAR - 6, catY + i * 22 + 12, 0x4400E5FF);
            }
            int color = i == selectedCategory ? UiTheme.accent() : UiTheme.textDim();
            ctx.drawText(textRenderer, Text.literal(CATEGORIES[i]), panelX + 14, catY + i * 22, color, true);
        }
    }

    private void goBack() {
        ModernConfig.save();
        client.setScreen(parent != null ? parent : new CustomTitleScreen());
    }

    private static final class ModCard {
        int category;
        String label;
        String action;
        String openTarget;
        BooleanSupplier getter;
        Consumer<Boolean> setter;
    }
}
