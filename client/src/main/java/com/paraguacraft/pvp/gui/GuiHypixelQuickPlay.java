package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.core.HypixelHelper;
import com.paraguacraft.pvp.core.ModLang;
import com.paraguacraft.pvp.gui.theme.UiTheme;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.init.Blocks;
import net.minecraft.init.Items;
import net.minecraft.item.Item;
import net.minecraft.item.ItemStack;
import org.lwjgl.input.Keyboard;
import org.lwjgl.input.Mouse;

import java.io.IOException;
import java.util.ArrayList;
import java.util.List;

/**
 * Juego rapido de Hypixel — menu con mini-juegos e iconos de items Minecraft.
 */
public class GuiHypixelQuickPlay extends GuiScreen {

    private static final int CARD_W = 200;
    private static final int CARD_H = 44;
    private static final int GAP = 8;
    private static final int ICON = 32;

    private static final class GameEntry {
        final String name;
        final String subtitle;
        final String command;
        final boolean lobbyOnly;
        final ItemStack icon;

        GameEntry(String name, String subtitle, String command, boolean lobbyOnly, ItemStack icon) {
            this.name = name;
            this.subtitle = subtitle;
            this.command = command;
            this.lobbyOnly = lobbyOnly;
            this.icon = icon;
        }
    }

    private static final GameEntry[] GAMES = {
        new GameEntry("Bed Wars", "Select Mode", "/play bedwars_eight_one", false, new ItemStack(Item.getItemFromBlock(Blocks.bed))),
        new GameEntry("SkyWars", "Select Mode", "/play solo_normal", false, new ItemStack(Blocks.grass)),
        new GameEntry("Arcade", "Select Mode", "/play arcade_mini_walls", false, new ItemStack(Items.golden_sword)),
        new GameEntry("Warlords", "Select Mode", "/play warlords_ctf", false, new ItemStack(Items.iron_chestplate)),
        new GameEntry("TNT Games", "Select Mode", "/play tnt_tnt_run", false, new ItemStack(Blocks.tnt)),
        new GameEntry("The Pit", "> Go To Lobby", "/lobby pit", true, new ItemStack(Items.gold_ingot)),
        new GameEntry("Build Battle", "Select Mode", "/play build_battle_speed_builders", false, new ItemStack(Items.brick)),
        new GameEntry("UHC", "Select Mode", "/play uhc_solo", false, new ItemStack(Items.golden_apple)),
        new GameEntry("Murder Mystery", "Select Mode", "/play murder_classic", false, new ItemStack(Items.bow)),
        new GameEntry("Tournament Hall", "> Go To Lobby", "/lobby tournaments", true, new ItemStack(Items.nether_star)),
        new GameEntry("Wool Games", "Select Mode", "/play wool_wool_wars", false, new ItemStack(Blocks.wool, 1, 11)),
        new GameEntry("Prototype", "Select Mode", "/lobby prototype", true, new ItemStack(Items.comparator)),
    };

    private GameEntry pendingModes;
    private final List<String> modeCommands = new ArrayList<String>();
    private float scroll;

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        drawRect(0, 0, width, height, 0x99000000);

        int panelW = Math.min(width - 40, 440);
        int panelH = Math.min(height - 60, 380);
        int panelX = width / 2 - panelW / 2;
        int panelY = height / 2 - panelH / 2;
        Gui.drawRect(panelX, panelY, panelX + panelW, panelY + panelH, 0xCC0A0C14);
        Gui.drawRect(panelX, panelY, panelX + panelW, panelY + 1, 0x4400E5FF);

        FontRenderer fr = fontRendererObj;
        fr.drawStringWithShadow(ModLang.format("paraguacraft.quickplay.title"), panelX + 16, panelY + 12, UiTheme.ACCENT);
        String hint = HypixelHelper.isOnHypixel()
            ? ModLang.format("paraguacraft.quickplay.hypixel")
            : ModLang.format("paraguacraft.quickplay.offline");
        fr.drawStringWithShadow(hint, panelX + 16, panelY + 26, UiTheme.TEXT_DIM);
        fr.drawStringWithShadow(ModLang.format("paraguacraft.quickplay.safe"), panelX + 16, panelY + 38, 0xFF88AA88);

        if (pendingModes != null) {
            drawModePanel(panelX, panelY, panelW, panelH, mouseX, mouseY);
        } else {
            drawGameGrid(panelX, panelY, panelW, panelH, mouseX, mouseY);
        }

        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    private void drawGameGrid(int panelX, int panelY, int panelW, int panelH, int mouseX, int mouseY) {
        FontRenderer fr = fontRendererObj;
        int cols = 2;
        int startX = panelX + 16;
        int startY = panelY + 52;
        int rows = (GAMES.length + cols - 1) / cols;
        int contentH = rows * (CARD_H + GAP);
        int viewH = panelH - 64;
        int maxScroll = Math.max(0, contentH - viewH);
        scroll = Math.max(0f, Math.min(maxScroll, scroll));

        for (int i = 0; i < GAMES.length; i++) {
            GameEntry game = GAMES[i];
            int col = i % cols;
            int row = i / cols;
            int x = startX + col * (CARD_W + GAP);
            int y = startY + row * (CARD_H + GAP) - (int) scroll;
            if (y + CARD_H < startY || y > startY + viewH) {
                continue;
            }
            boolean hover = mouseX >= x && mouseX <= x + CARD_W && mouseY >= y && mouseY <= y + CARD_H;
            Gui.drawRect(x, y, x + CARD_W, y + CARD_H, hover ? 0xCC1A2030 : 0xAA121820);
            Gui.drawRect(x, y, x + 46, y + CARD_H, 0x3300E5FF);
            drawIcon(game.icon, x + 7, y + 6);
            fr.drawStringWithShadow(game.name, x + 50, y + 12, UiTheme.TEXT);
            fr.drawStringWithShadow(game.subtitle, x + 50, y + 26, UiTheme.TEXT_DIM);
        }
    }

    private void drawIcon(ItemStack stack, int x, int y) {
        if (stack == null) {
            return;
        }
        GlStateManager.pushMatrix();
        GlStateManager.enableDepth();
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
        mc.getRenderItem().renderItemAndEffectIntoGUI(stack, x, y);
        GlStateManager.disableDepth();
        GlStateManager.disableLighting();
        GlStateManager.popMatrix();
    }

    private void drawModePanel(int panelX, int panelY, int panelW, int panelH, int mouseX, int mouseY) {
        FontRenderer fr = fontRendererObj;
        drawIcon(pendingModes.icon, panelX + 16, panelY + 46);
        fr.drawStringWithShadow(pendingModes.name, panelX + 52, panelY + 52, UiTheme.TEXT);
        int y = panelY + 78;
        for (int i = 0; i < modeCommands.size(); i++) {
            String mode = modeCommands.get(i);
            int rowY = y + i * 22;
            boolean hover = mouseX >= panelX + 16 && mouseX <= panelX + panelW - 16
                && mouseY >= rowY && mouseY <= rowY + 18;
            Gui.drawRect(panelX + 16, rowY, panelX + panelW - 16, rowY + 18, hover ? 0xFF334455 : 0xFF223344);
            fr.drawStringWithShadow(mode, panelX + 24, rowY + 5, UiTheme.TEXT);
        }
        int backY = panelY + panelH - 28;
        boolean backHover = mouseX >= panelX + 16 && mouseX <= panelX + 80 && mouseY >= backY && mouseY <= backY + 18;
        Gui.drawRect(panelX + 16, backY, panelX + 80, backY + 18, backHover ? UiTheme.ACCENT : 0xFF226688);
        fr.drawStringWithShadow("<", panelX + 24, backY + 5, 0xFFFFFF);
    }

    @Override
    protected void mouseClicked(int mouseX, int mouseY, int mouseButton) throws IOException {
        if (mouseButton != 0) {
            return;
        }
        int panelW = Math.min(width - 40, 440);
        int panelH = Math.min(height - 60, 380);
        int panelX = width / 2 - panelW / 2;
        int panelY = height / 2 - panelH / 2;

        if (pendingModes != null) {
            int backY = panelY + panelH - 28;
            if (mouseX >= panelX + 16 && mouseX <= panelX + 80 && mouseY >= backY && mouseY <= backY + 18) {
                pendingModes = null;
                modeCommands.clear();
                return;
            }
            int y = panelY + 78;
            for (int i = 0; i < modeCommands.size(); i++) {
                int rowY = y + i * 22;
                if (mouseX >= panelX + 16 && mouseX <= panelX + panelW - 16
                    && mouseY >= rowY && mouseY <= rowY + 18) {
                    HypixelHelper.sendCommand(modeCommands.get(i));
                    mc.displayGuiScreen(null);
                    return;
                }
            }
            return;
        }

        int cols = 2;
        int startX = panelX + 16;
        int startY = panelY + 52;
        for (int i = 0; i < GAMES.length; i++) {
            GameEntry game = GAMES[i];
            int col = i % cols;
            int row = i / cols;
            int x = startX + col * (CARD_W + GAP);
            int y = startY + row * (CARD_H + GAP) - (int) scroll;
            if (mouseX >= x && mouseX <= x + CARD_W && mouseY >= y && mouseY <= y + CARD_H) {
                if (game.lobbyOnly || game.subtitle.startsWith(">")) {
                    HypixelHelper.sendCommand(game.command);
                    mc.displayGuiScreen(null);
                } else {
                    openModes(game);
                }
                return;
            }
        }
    }

    @Override
    public void handleMouseInput() throws IOException {
        super.handleMouseInput();
        if (pendingModes != null) {
            return;
        }
        int wheel = Mouse.getEventDWheel();
        if (wheel != 0) {
            scroll -= wheel * 0.15f;
        }
    }

    private void openModes(GameEntry game) {
        pendingModes = game;
        modeCommands.clear();
        if (game.name.equals("Bed Wars")) {
            modeCommands.add("/play bedwars_eight_one");
            modeCommands.add("/play bedwars_four_four");
            modeCommands.add("/play bedwars_two_four");
            modeCommands.add("/play bedwars_capture");
            modeCommands.add("/lobby bedwars");
        } else if (game.name.equals("SkyWars")) {
            modeCommands.add("/play solo_normal");
            modeCommands.add("/play teams_normal");
            modeCommands.add("/play solo_insane");
            modeCommands.add("/play teams_insane");
            modeCommands.add("/lobby skywars");
        } else {
            modeCommands.add(game.command);
            modeCommands.add("/lobby " + game.name.toLowerCase().replace(" ", ""));
        }
    }

    @Override
    protected void keyTyped(char typedChar, int keyCode) throws IOException {
        if (keyCode == Keyboard.KEY_ESCAPE) {
            if (pendingModes != null) {
                pendingModes = null;
                modeCommands.clear();
                return;
            }
            mc.displayGuiScreen(null);
        }
    }

    @Override
    public boolean doesGuiPauseGame() {
        return false;
    }
}
