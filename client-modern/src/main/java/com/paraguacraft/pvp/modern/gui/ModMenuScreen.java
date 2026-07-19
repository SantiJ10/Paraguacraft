package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.PerformanceConfig;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.Click;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.function.BooleanSupplier;
import java.util.function.Consumer;

/** Mod Menu estilo Lunar / Paraguacraft 1.8.9 con categorias y busqueda. */
public class ModMenuScreen extends Screen {

    private static final String[] CATEGORIES = {"Todos", "HUD", "PvP", "Mecanicas", "Rendimiento", "Hypixel"};
    private static final int SIDEBAR = 132;
    private static final int TOPBAR = 44;

    private final Screen parent;
    private int selectedCategory;
    private String search = "";
    private float scroll;

    public ModMenuScreen(Screen parent) {
        super(Text.literal("Paraguacraft Mods"));
        this.parent = parent;
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
        addDrawableChild(ButtonWidget.builder(Text.literal("Editar HUD"), b ->
            client.setScreen(new GuiEditHudScreen(this))).dimensions(panelX + 12, closeY, 100, 20).build());
        addDrawableChild(ButtonWidget.builder(Text.literal("Cerrar"), b -> goBack())
            .dimensions(panelX + panelW - 112, closeY, 100, 20).build());
    }

    private void addCardButton(ModCard card, int x, int y, int w, int h) {
        if ("open".equals(card.action)) {
            addDrawableChild(ButtonWidget.builder(Text.literal(card.label), b -> open(card.openTarget))
                .dimensions(x, y, w, h).build());
        } else {
            addDrawableChild(ButtonWidget.builder(Text.literal(toggleLabel(card.label, card.getter.getAsBoolean())), b -> {
                card.setter.accept(!card.getter.getAsBoolean());
                ModernConfig.save();
                clearChildren();
                init();
            }).dimensions(x, y, w, h).build());
        }
    }

    private void open(String target) {
        switch (target) {
            case "quickplay" -> client.setScreen(new HypixelQuickPlayScreen(this));
            case "packs" -> client.setScreen(new PackSelectScreen(this));
            case "theme" -> client.setScreen(new ThemeSelectScreen(this));
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
        cards.add(toggle(1, "Musica (Spotify/YT)", () -> ModernConfig.showMusicHud, v -> ModernConfig.showMusicHud = v));
        cards.add(toggle(1, "Pociones HUD", () -> ModernConfig.showPotions, v -> ModernConfig.showPotions = v));
        cards.add(toggle(1, "Brújula", () -> ModernConfig.showCompass, v -> ModernConfig.showCompass = v));
        cards.add(toggle(1, "Combo counter", () -> ModernConfig.comboCounter, v -> ModernConfig.comboCounter = v));
        cards.add(toggle(2, "Hitbox azul", () -> ModernConfig.showBlockOutline, v -> ModernConfig.showBlockOutline = v));
        cards.add(toggle(2, "No hurt cam", () -> ModernConfig.noHurtCam, v -> ModernConfig.noHurtCam = v));
        cards.add(toggle(2, "Low fire", () -> ModernConfig.lowFire, v -> ModernConfig.lowFire = v));
        cards.add(toggle(2, "Item physics", () -> ModernConfig.itemPhysics, v -> ModernConfig.itemPhysics = v));
        cards.add(toggle(2, "TNT countdown", () -> ModernConfig.showTntCountdown, v -> ModernConfig.showTntCountdown = v));
        cards.add(toggle(3, "Toggle sprint (W)", () -> ModernConfig.toggleSprintLegacy, v -> ModernConfig.toggleSprintLegacy = v));
        cards.add(toggle(3, "Fullbright", () -> ModernConfig.fullbright, v -> ModernConfig.fullbright = v));
        cards.add(toggle(3, "FOV dinamico", () -> ModernConfig.dynamicFov, v -> ModernConfig.dynamicFov = v));
        cards.add(toggle(3, "Freelook (Alt)", () -> ModernConfig.freelookEnabled, v -> ModernConfig.freelookEnabled = v));
        cards.add(toggle(3, "Old animations", () -> ModernConfig.oldAnimations, v -> ModernConfig.oldAnimations = v));
        cards.add(toggle(3, "Ocultar titulos", () -> ModernConfig.hideTitles, v -> ModernConfig.hideTitles = v));
        cards.add(toggle(3, "Chat triggers", () -> ModernConfig.chatTriggers, v -> ModernConfig.chatTriggers = v));
        cards.add(toggle(3, "Scoreboard", () -> ModernConfig.scoreboardEnabled, v -> ModernConfig.scoreboardEnabled = v));
        cards.add(toggle(4, "Boost FPS", () -> PerformanceConfig.boostFps, v -> PerformanceConfig.boostFps = v));
        cards.add(open(5, "Hypixel Quick Play", "quickplay"));
        cards.add(open(5, "Texture packs", "packs"));
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
    public boolean keyPressed(net.minecraft.client.input.KeyInput input) {
        return super.keyPressed(input);
    }

    @Override
    public void render(DrawContext ctx, int mouseX, int mouseY, float delta) {
        ctx.fill(0, 0, width, height, 0x99000000);
        int panelX = Math.max(8, width / 2 - 420);
        int panelY = 20;
        int panelW = Math.min(width - 16, 840);
        int panelH = height - 40;
        ctx.fill(panelX, panelY, panelX + panelW, panelY + panelH, 0xE00A0C14);
        ctx.fill(panelX, panelY, panelX + panelW, panelY + 1, 0x33FFFFFF);

        ctx.drawText(textRenderer, Text.literal("PARAGUACRAFT"), panelX + 14, panelY + 12, UiTheme.accent(), true);
        ctx.drawText(textRenderer, Text.literal("Mod Menu"), panelX + 14, panelY + 26, UiTheme.textDim(), true);

        ctx.fill(panelX, panelY + TOPBAR, panelX + SIDEBAR, panelY + panelH, 0xDD080A10);
        int catY = panelY + TOPBAR + 8;
        for (int i = 0; i < CATEGORIES.length; i++) {
            int color = i == selectedCategory ? UiTheme.accent() : UiTheme.textDim();
            ctx.drawText(textRenderer, Text.literal(CATEGORIES[i]), panelX + 14, catY + i * 22, color, true);
        }

        ctx.drawText(textRenderer, Text.literal("Buscar: " + search), panelX + SIDEBAR + 12, panelY + 16, UiTheme.textDim(), true);
        super.render(ctx, mouseX, mouseY, delta);
    }

    private void goBack() {
        ModernConfig.save();
        client.setScreen(parent != null ? parent : null);
    }

    private static String toggleLabel(String label, boolean on) {
        return label + "\n" + (on ? "ON" : "OFF");
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
