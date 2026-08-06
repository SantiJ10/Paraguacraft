package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.core.PvpServers;
import com.paraguacraft.pvp.gui.theme.UiTheme;
import java.util.List;
import net.minecraft.client.gui.GuiButton;
import net.minecraft.client.gui.GuiMultiplayer;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.multiplayer.ServerData;
import net.minecraft.client.multiplayer.ServerList;
import net.minecraftforge.fml.client.FMLClientHandler;

/**
 * Menú de redirección rápida PvP (solo cliente Paraguacraft 1.8.9).
 * Layout alineado con el multijugador modern: lista de servers + vanilla + volver.
 */
public class GuiPvpServerSelect extends GuiScreen {

    private final GuiScreen parent;
    private List<PvpServers.Entry> servers;

    public GuiPvpServerSelect(GuiScreen parent) {
        this.parent = parent;
    }

    @Override
    public void initGui() {
        this.servers = PvpServers.list();
        this.buttonList.clear();
        int btnW = 220;
        int btnH = 24;
        int startY = 56;
        int gap = 26;
        for (int i = 0; i < servers.size(); i++) {
            PvpServers.Entry e = servers.get(i);
            this.buttonList.add(new EasingButton(
                100 + i,
                this.width / 2 - btnW / 2,
                startY + i * gap,
                btnW,
                btnH,
                e.name
            ));
        }
        int after = startY + servers.size() * gap + 10;
        this.buttonList.add(new EasingButton(1, this.width / 2 - btnW / 2, after, btnW, btnH, "Lista vanilla / LAN"));
        this.buttonList.add(new EasingButton(0, this.width / 2 - btnW / 2, after + gap, btnW, btnH, "Volver"));
    }

    @Override
    protected void actionPerformed(GuiButton button) {
        if (button.id == 0) {
            this.mc.displayGuiScreen(this.parent);
            return;
        }
        if (button.id == 1) {
            this.mc.displayGuiScreen(new GuiMultiplayer(this));
            return;
        }
        int idx = button.id - 100;
        if (idx >= 0 && idx < servers.size()) {
            connect(servers.get(idx));
        }
    }

    private void connect(PvpServers.Entry entry) {
        String address = PvpServers.normalizeAddress(entry.address);
        ServerData data = new ServerData(entry.name, address, false);
        // Guardar en servers.dat (best effort) para icono/lista vanilla.
        try {
            ServerList list = new ServerList(this.mc);
            list.loadServerList();
            boolean found = false;
            for (int i = 0; i < list.countServers(); i++) {
                ServerData existing = list.getServerData(i);
                if (existing != null && address.equalsIgnoreCase(existing.serverIP)) {
                    found = true;
                    break;
                }
            }
            if (!found) {
                list.addServerData(data);
                list.saveServerList();
            }
        } catch (Exception ignored) {
        }
        FMLClientHandler.instance().connectToServer(this, data);
    }

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        PanoramaBackground.draw(this, partialTicks);
        this.drawCenteredString(this.fontRendererObj, "Servidores PvP", this.width / 2, 32, UiTheme.ACCENT);
        super.drawScreen(mouseX, mouseY, partialTicks);
    }
}
