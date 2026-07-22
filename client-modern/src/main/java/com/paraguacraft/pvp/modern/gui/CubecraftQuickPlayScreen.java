package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.core.CubecraftHelper;
import com.paraguacraft.pvp.modern.core.QuickPlayState;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;

/** Cubecraft Quick Play — reconectar y elegir modo. */
public class CubecraftQuickPlayScreen extends ParaguacraftScreen {

    private record GameEntry(String name, String command) {}

    private static final GameEntry[] GAMES = {
        new GameEntry("EggWars Solo", "server EggWars-Solo"),
        new GameEntry("SkyWars Solo", "server SkyWars-Solo"),
        new GameEntry("Lucky Islands", "server LuckyIslands-Solo"),
        new GameEntry("BlockWars Solo", "server BlockWars-Solo"),
        new GameEntry("Survival Games", "server SurvivalGames-Solo"),
        new GameEntry("PvP", "server PvP"),
    };

    public CubecraftQuickPlayScreen(Screen parent) {
        super(Text.literal("Cubecraft Quick Play"), parent);
    }

    @Override
    protected void init() {
        int btnW = 240;
        int btnH = 22;
        int startY = 64;
        int gap = 24;
        int i = 0;

        if (QuickPlayState.hasLastFor(QuickPlayState.TargetServer.CUBECRAFT)) {
            addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, startY, btnW, btnH,
                Text.literal("Reconectar: " + QuickPlayState.lastLabel),
                () -> QuickPlayState.reconnect(client, QuickPlayState.TargetServer.CUBECRAFT)));
            startY += gap;
            i++;
        }

        for (GameEntry game : GAMES) {
            int y = startY + i * gap;
            addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, y, btnW, btnH,
                Text.literal(game.name()), () -> play(game)));
            i++;
        }

        int after = startY + i * gap + 4;
        addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, after, btnW, btnH,
            Text.literal("Volver"), () -> client.setScreen(parent)));
    }

    private void play(GameEntry game) {
        QuickPlayState.remember(QuickPlayState.TargetServer.CUBECRAFT, game.command(), game.name());
        if (CubecraftHelper.isOnCubecraft(client)) {
            CubecraftHelper.sendCommand(client, game.command());
            client.setScreen(null);
        } else {
            QuickPlayState.queue(QuickPlayState.TargetServer.CUBECRAFT, game.command());
            CubecraftHelper.connect(client, parent != null ? parent : this);
        }
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        super.render(context, mouseX, mouseY, delta);
        String hint = CubecraftHelper.isOnCubecraft(client)
            ? "Conectado a Cubecraft"
            : "Conectate a Cubecraft para jugar";
        context.drawCenteredTextWithShadow(textRenderer, Text.literal("Cubecraft Quick Play"), width / 2, 40, UiTheme.accent());
        context.drawCenteredTextWithShadow(textRenderer, Text.literal(hint), width / 2, 52, UiTheme.textDim());
    }
}
