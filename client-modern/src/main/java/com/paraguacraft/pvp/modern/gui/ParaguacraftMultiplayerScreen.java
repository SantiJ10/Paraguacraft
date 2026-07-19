package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import com.paraguacraft.pvp.modern.network.DefaultServers;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.multiplayer.ConnectScreen;
import net.minecraft.client.gui.screen.multiplayer.MultiplayerScreen;
import net.minecraft.client.network.ServerAddress;
import net.minecraft.client.network.ServerInfo;
import net.minecraft.text.Text;

import java.util.List;

/** Multijugador con servidores PvP predefinidos. */
public class ParaguacraftMultiplayerScreen extends ParaguacraftScreen {

    private List<DefaultServers.Entry> servers;

    public ParaguacraftMultiplayerScreen(Screen parent) {
        super(Text.literal("Multijugador PvP"), parent);
    }

    @Override
    protected void init() {
        servers = DefaultServers.load(client);
        int btnW = 260;
        int btnH = 24;
        int startY = 72;
        int gap = 26;
        for (int i = 0; i < servers.size(); i++) {
            DefaultServers.Entry entry = servers.get(i);
            int y = startY + i * gap;
            addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, y, btnW, btnH,
                Text.literal(entry.name()), () -> connect(entry)));
        }
        int after = startY + servers.size() * gap + 8;
        addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, after, btnW, btnH,
            Text.literal("Lista vanilla / LAN"), () -> client.setScreen(new MultiplayerScreen(this))));
        addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, after + gap, btnW, btnH,
            Text.literal("Volver"), () -> client.setScreen(parent)));
    }

    private void connect(DefaultServers.Entry entry) {
        ServerInfo info = new ServerInfo(entry.name(), entry.address(), ServerInfo.ServerType.OTHER);
        ConnectScreen.connect(this, client, ServerAddress.parse(entry.address()), info, false, null);
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        super.render(context, mouseX, mouseY, delta);
        context.drawCenteredTextWithShadow(
            textRenderer,
            Text.literal("Servidores PvP"),
            width / 2,
            48,
            UiTheme.accent()
        );
    }
}
