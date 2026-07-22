package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.core.GameModeDetector;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;

/** Override manual del modo de juego detectado. */
public class GuiGameModeOverrideScreen extends ParaguacraftScreen {

    private static final GameModeDetector.Mode[] OPTIONS = {
        GameModeDetector.Mode.AUTO,
        GameModeDetector.Mode.LOBBY,
        GameModeDetector.Mode.BEDWARS,
        GameModeDetector.Mode.SKYWARS,
        GameModeDetector.Mode.DUELS,
        GameModeDetector.Mode.BUILD_BATTLE,
        GameModeDetector.Mode.TNT_RUN,
        GameModeDetector.Mode.LUCKY_ISLANDS,
        GameModeDetector.Mode.PVP,
        GameModeDetector.Mode.OTHER,
    };

    public GuiGameModeOverrideScreen(Screen parent) {
        super(Text.literal("Modo de juego"), parent);
    }

    @Override
    protected void init() {
        int btnW = 240;
        int btnH = 20;
        int startY = 58;
        int gap = 22;
        for (int i = 0; i < OPTIONS.length; i++) {
            GameModeDetector.Mode mode = OPTIONS[i];
            int y = startY + i * gap;
            boolean selected = isSelected(mode);
            String prefix = selected ? "> " : "  ";
            String label = mode == GameModeDetector.Mode.AUTO
                ? prefix + "Auto (scoreboard)"
                : prefix + GameModeDetector.labelFor(mode);
            addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, y, btnW, btnH,
                Text.literal(label), () -> {
                    GameModeDetector.setManualOverride(mode);
                    client.setScreen(parent);
                }));
        }
        int after = startY + OPTIONS.length * gap + 8;
        addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, after, btnW, btnH,
            Text.literal("Volver"), () -> client.setScreen(parent)));
    }

    private static boolean isSelected(GameModeDetector.Mode mode) {
        if (mode == GameModeDetector.Mode.AUTO) {
            return !GameModeDetector.isManualOverride();
        }
        return GameModeDetector.isManualOverride() && GameModeDetector.current() == mode;
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        super.render(context, mouseX, mouseY, delta);
        context.drawCenteredTextWithShadow(textRenderer, Text.literal("Modo de juego"), width / 2, 36, UiTheme.accent());
        context.drawCenteredTextWithShadow(
            textRenderer,
            Text.literal("Detectado: " + GameModeDetector.labelFor(GameModeDetector.detectedMode())),
            width / 2,
            48,
            UiTheme.textDim()
        );
    }
}
