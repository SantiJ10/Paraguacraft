package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.NickFinderManager;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.Click;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.TextFieldWidget;
import net.minecraft.client.network.PlayerListEntry;
import net.minecraft.text.Text;

import java.util.List;

/** Busca nicks en el tab (estilo Hytils NickFinder). */
public class NickFinderScreen extends ParaguacraftScreen {

    private TextFieldWidget searchField;
    private List<PlayerListEntry> matches = List.of();

    public NickFinderScreen(Screen parent) {
        super(Text.literal("NickFinder"), parent);
    }

    @Override
    protected void init() {
        int panelW = 280;
        int cx = width / 2;
        searchField = new TextFieldWidget(textRenderer, cx - panelW / 2, height / 2 - 90, panelW, 20, Text.literal("Buscar"));
        searchField.setMaxLength(32);
        searchField.setText(NickFinderManager.query());
        searchField.setChangedListener(text -> {
            NickFinderManager.setQuery(text);
            refreshMatches();
            ModernConfig.save();
        });
        searchField.setFocused(true);
        addDrawableChild(searchField);
        refreshMatches();
        addDrawableChild(FlatMenuButton.create(cx - panelW / 2, height / 2 + 90, panelW, 20,
            Text.literal("Cerrar"), this::close));
    }

    private void refreshMatches() {
        matches = NickFinderManager.findEntries();
    }

    @Override
    public void render(DrawContext ctx, int mouseX, int mouseY, float delta) {
        renderBackground(ctx, mouseX, mouseY, delta);
        int panelW = 280;
        int cx = width / 2;
        int py = height / 2 - 110;
        ctx.fill(cx - panelW / 2 - 8, py, cx + panelW / 2 + 8, height / 2 + 120, 0xCC0A0C14);
        ctx.drawCenteredTextWithShadow(textRenderer, Text.literal("NickFinder — Tab list"), cx, py + 8, UiTheme.accent());
        ctx.drawCenteredTextWithShadow(textRenderer,
            Text.literal("Coincidencias parciales resaltadas en tab"),
            cx, py + 22, UiTheme.textDim());
        int y = height / 2 - 58;
        if (NickFinderManager.query().length() < 2) {
            ctx.drawCenteredTextWithShadow(textRenderer, Text.literal("Escribí al menos 2 letras…"), cx, y, UiTheme.textDim());
        } else if (matches.isEmpty()) {
            ctx.drawCenteredTextWithShadow(textRenderer, Text.literal("Sin coincidencias"), cx, y, UiTheme.textDim());
        } else {
            int max = Math.min(8, matches.size());
            for (int i = 0; i < max; i++) {
                PlayerListEntry entry = matches.get(i);
                String name = entry.getProfile().name();
                int ping = entry.getLatency();
                int color = UiTheme.accent();
                ctx.drawText(textRenderer, Text.literal(name), cx - panelW / 2, y + i * 14, color, true);
                String pingText = ping + " ms";
                ctx.drawText(textRenderer, Text.literal(pingText),
                    cx + panelW / 2 - textRenderer.getWidth(pingText), y + i * 14, UiTheme.textDim(), true);
            }
            if (matches.size() > max) {
                ctx.drawCenteredTextWithShadow(textRenderer,
                    Text.literal("+" + (matches.size() - max) + " más…"),
                    cx, y + max * 14 + 4, UiTheme.textDim());
            }
        }
        super.render(ctx, mouseX, mouseY, delta);
    }

    @Override
    public boolean mouseClicked(Click click, boolean doubled) {
        setFocused(searchField);
        return super.mouseClicked(click, doubled);
    }
}
