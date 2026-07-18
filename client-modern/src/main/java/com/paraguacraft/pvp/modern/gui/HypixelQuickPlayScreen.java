package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.core.HypixelHelper;
import com.paraguacraft.pvp.modern.core.QuickPlayState;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

/** Hypixel Quick Play — reconectar y elegir modo. */
public class HypixelQuickPlayScreen extends Screen {

    private record GameEntry(String name, String command) {}

    private static final GameEntry[] GAMES = {
        new GameEntry("Bed Wars", "play bedwars_eight_one"),
        new GameEntry("SkyWars Solo", "play solo_normal"),
        new GameEntry("Duels", "play duels_classic_duel"),
        new GameEntry("The Pit", "lobby pit"),
        new GameEntry("Arcade Mini Walls", "play arcade_mini_walls"),
        new GameEntry("Murder Mystery", "play murder_classic"),
        new GameEntry("Build Battle", "play build_battle_speed_builders"),
        new GameEntry("Wool Wars", "play wool_wool_wars"),
    };

    private final Screen parent;

    public HypixelQuickPlayScreen(Screen parent) {
        super(Text.literal("Hypixel Quick Play"));
        this.parent = parent;
    }

    @Override
    protected void init() {
        int btnW = 240;
        int btnH = 22;
        int startY = 64;
        int gap = 24;
        int i = 0;

        if (QuickPlayState.hasLast()) {
            addDrawableChild(ButtonWidget.builder(
                Text.literal("Reconectar: " + QuickPlayState.lastLabel),
                b -> QuickPlayState.reconnect(client)
            ).dimensions(width / 2 - btnW / 2, startY, btnW, btnH).build());
            startY += gap;
            i++;
        }

        for (GameEntry game : GAMES) {
            int y = startY + i * gap;
            addDrawableChild(ButtonWidget.builder(Text.literal(game.name()), b -> play(game)).dimensions(width / 2 - btnW / 2, y, btnW, btnH).build());
            i++;
        }

        int after = startY + i * gap + 4;
        addDrawableChild(ButtonWidget.builder(Text.literal("Volver"), b ->
            client.setScreen(parent)).dimensions(width / 2 - btnW / 2, after, btnW, btnH).build());
    }

    private void play(GameEntry game) {
        QuickPlayState.remember(game.command(), game.name());
        if (HypixelHelper.isOnHypixel(client)) {
            HypixelHelper.sendCommand(client, game.command());
            client.setScreen(null);
        } else {
            QuickPlayState.queue(game.command());
            HypixelHelper.connect(client, parent != null ? parent : this);
        }
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        MenuBackground.draw(this, context, mouseX, mouseY, delta);
        String hint = HypixelHelper.isOnHypixel(client)
            ? "Conectado a Hypixel"
            : "Conectate a Hypixel para jugar";
        context.drawCenteredTextWithShadow(textRenderer, Text.literal("Hypixel Quick Play"), width / 2, 40, UiTheme.accent());
        context.drawCenteredTextWithShadow(textRenderer, Text.literal(hint), width / 2, 52, UiTheme.textDim());
        super.render(context, mouseX, mouseY, delta);
    }
}
