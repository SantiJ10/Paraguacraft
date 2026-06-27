package com.paraguacraft.pvp.hud;

import com.paraguacraft.pvp.core.LauncherIpc;
import com.paraguacraft.pvp.gui.theme.TextUtil;
import com.paraguacraft.pvp.gui.theme.UiTheme;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.resources.I18n;
import net.minecraft.init.Items;
import net.minecraft.item.ItemStack;
import net.minecraft.util.ResourceLocation;
import org.lwjgl.opengl.GL11;

/** HUDs M4: launcher overlay, Bedwars resources. */
public final class AdvancedHud {

    private static final int BW_PANEL_W = 108;
    private static final int BW_PANEL_H = 52;
    private static final int ART_SIZE = 16;

    private AdvancedHud() {}

    public static int bwPanelW() {
        return BW_PANEL_W;
    }

    public static int bwPanelH() {
        return BW_PANEL_H;
    }

    public static int overlayPanelHeight(LauncherIpc.Snapshot snap) {
        int lines = 0;
        if (ModConfig.showHardwareHud && snap.valid) {
            lines += 3;
            if (snap.gpuPct >= 0f) {
                lines++;
            }
            if (snap.tempC >= 0f) {
                lines++;
            }
        }
        if (ModConfig.showMusicHud && snap.valid && snap.musicPlaying) {
            lines += 2;
        }
        if (lines == 0) {
            return 0;
        }
        int h = 8 + lines * 11;
        if (ModConfig.showMusicHud && snap.valid && snap.musicPlaying
            && ModConfig.showMusicAlbumArt && !snap.musicImageUrl.isEmpty()) {
            h = Math.max(h, 8 + ART_SIZE + 2);
        }
        return h;
    }

    public static int overlayPanelWidth(LauncherIpc.Snapshot snap) {
        int w = ModConfig.overlayHudW;
        if (ModConfig.showMusicHud && snap.valid && snap.musicPlaying) {
            String title = snap.musicTitle.isEmpty() ? "-" : snap.musicTitle;
            int textW = HudDraw.width(trunc(title, 22)) + 12;
            if (ModConfig.showMusicAlbumArt && !snap.musicImageUrl.isEmpty()) {
                textW += ART_SIZE + 6;
            }
            w = Math.max(w, textW);
        }
        return w;
    }

    public static void drawOverlay() {
        if (!ModConfig.showHardwareHud && !ModConfig.showMusicHud) {
            return;
        }
        LauncherIpc.Snapshot snap = LauncherIpc.get();
        if (!snap.valid && !ModConfig.showMusicHud) {
            return;
        }
        int lines = 0;
        if (ModConfig.showHardwareHud && snap.valid) {
            lines += 3;
            if (snap.gpuPct >= 0f) {
                lines++;
            }
            if (snap.tempC >= 0f) {
                lines++;
            }
        }
        if (ModConfig.showMusicHud && snap.valid && snap.musicPlaying) {
            lines += 2;
        }
        if (lines == 0) {
            return;
        }

        int x = ModConfig.overlayHudX;
        int y = ModConfig.overlayHudY;
        int w = overlayPanelWidth(snap);
        int h = overlayPanelHeight(snap);
        int bgAlpha = (ModConfig.showMusicHud && snap.valid && snap.musicPlaying)
            ? ModConfig.musicHudAlpha
            : 0x99;
        int bgColor = (bgAlpha << 24) | 0x0A0C14;

        GlStateManager.enableBlend();
        Gui.drawRect(x, y, x + w, y + h, bgColor);
        Gui.drawRect(x, y, x + w, y + 1, 0x4400E5FF);

        int ty = y + 6;
        if (ModConfig.showHardwareHud && snap.valid) {
            HudDraw.labeled("CPU ", fmtPct(snap.cpuPct), x + 6, ty);
            ty += 11;
            HudDraw.labeled("RAM ", fmtPct(snap.ramPct), x + 6, ty);
            ty += 11;
            if (snap.gpuPct >= 0f) {
                HudDraw.labeled("GPU ", fmtPct(snap.gpuPct), x + 6, ty);
                ty += 11;
            }
            if (snap.tempC >= 0f) {
                HudDraw.labeled("TEMP ", String.format("%.0f C", snap.tempC), x + 6, ty);
                ty += 11;
            }
        }

        if (ModConfig.showMusicHud && snap.valid && snap.musicPlaying) {
            int textX = x + 6;
            boolean showArt = ModConfig.showMusicAlbumArt && !snap.musicImageUrl.isEmpty();
            if (showArt) {
                drawMusicArt(x + 6, ty, snap.musicImageUrl);
                textX = x + 6 + ART_SIZE + 4;
            }
            String title = snap.musicTitle.isEmpty() ? "-" : TextUtil.sanitizeForMcFont(snap.musicTitle);
            HudDraw.text(trunc(title, 22), textX, ty, UiTheme.ACCENT);
            ty += 11;
            if (!snap.musicArtist.isEmpty()) {
                HudDraw.text(trunc(TextUtil.sanitizeForMcFont(snap.musicArtist), 24), textX, ty, UiTheme.TEXT_DIM);
            }
        }
        GlStateManager.disableBlend();
    }

    private static void drawMusicArt(int x, int y, String imageUrl) {
        Minecraft mc = Minecraft.getMinecraft();
        ResourceLocation art = MusicArtCache.get(imageUrl);
        GlStateManager.pushMatrix();
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
        if (art == null) {
            GlStateManager.enableDepth();
            mc.getRenderItem().renderItemIntoGUI(new ItemStack(Items.record_13), x, y);
            GlStateManager.disableDepth();
            GlStateManager.disableLighting();
        } else {
            GlStateManager.enableBlend();
            GlStateManager.enableAlpha();
            GlStateManager.blendFunc(GL11.GL_SRC_ALPHA, GL11.GL_ONE_MINUS_SRC_ALPHA);
            mc.getTextureManager().bindTexture(art);
            Gui.drawScaledCustomSizeModalRect(x, y, 0, 0, 64, 64, ART_SIZE, ART_SIZE, 64, 64);
            GlStateManager.disableBlend();
        }
        GlStateManager.popMatrix();
    }

    public static void drawBedwarsResources(net.minecraft.entity.player.EntityPlayer player) {
        if (!ModConfig.showBedwarsResources || player == null) {
            return;
        }
        int iron = 0;
        int gold = 0;
        int diamond = 0;
        int emerald = 0;
        for (net.minecraft.item.ItemStack stack : player.inventory.mainInventory) {
            if (stack == null) {
                continue;
            }
            net.minecraft.item.Item item = stack.getItem();
            int n = stack.stackSize;
            if (item == Items.iron_ingot) {
                iron += n;
            } else if (item == Items.gold_ingot) {
                gold += n;
            } else if (item == Items.diamond) {
                diamond += n;
            } else if (item == Items.emerald) {
                emerald += n;
            }
        }
        int x = ModConfig.bwResX;
        int y = ModConfig.bwResY;
        GlStateManager.enableBlend();
        if (!ModConfig.bwResTransparentBg) {
            Gui.drawRect(x, y, x + BW_PANEL_W, y + BW_PANEL_H, 0x88000000);
        }
        drawBwLine(I18n.format("paraguacraft.hud.bw.iron"), iron, x + 6, y + 6);
        drawBwLine(I18n.format("paraguacraft.hud.bw.gold"), gold, x + 6, y + 18);
        drawBwLine(I18n.format("paraguacraft.hud.bw.emerald"), emerald, x + 6, y + 30);
        drawBwLine(I18n.format("paraguacraft.hud.bw.diamond"), diamond, x + 6, y + 42);
        GlStateManager.disableBlend();
    }

    private static void drawBwLine(String name, int count, int x, int y) {
        HudDraw.labeled(name + " ", String.valueOf(count), x, y);
    }

    private static String fmtPct(float v) {
        if (v < 0f) {
            return "-";
        }
        return String.format("%.0f%%", Math.min(100f, v));
    }

    private static String trunc(String s, int max) {
        if (s.length() <= max) {
            return s;
        }
        return s.substring(0, max - 1) + "...";
    }
}
