package com.paraguacraft.pvp.modern.hud;

import com.paraguacraft.pvp.modern.ParaguacraftPvPModern;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.PerformanceConfig;
import net.fabricmc.fabric.api.client.rendering.v1.hud.HudElementRegistry;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.font.TextRenderer;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.network.ClientPlayNetworkHandler;
import net.minecraft.client.network.PlayerListEntry;
import net.minecraft.client.render.RenderTickCounter;
import net.minecraft.text.Text;
import net.minecraft.util.Identifier;

/** HUD minimo: FPS, ping y keystrokes WASD + LMB/RMB. */
public final class HudRenderer {

    private static final Identifier HUD_ID = Identifier.of(ParaguacraftPvPModern.MOD_ID, "hud");

    private static long lastLeftClick;
    private static long lastRightClick;
    private static int leftCps;
    private static int rightCps;
    private static int leftTicks;
    private static int rightTicks;
    private static long cpsWindowStart;

    private HudRenderer() {}

    public static void register() {
        HudElementRegistry.addFirst(HUD_ID, HudRenderer::render);
    }

    private static void render(DrawContext context, RenderTickCounter tickCounter) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client == null || client.options.hudHidden || client.currentScreen != null) {
            return;
        }
        tickCps(client);
        TextRenderer tr = client.textRenderer;
        int x = ModernConfig.hudX;
        int y = ModernConfig.hudY;
        int line = 0;
        if (ModernConfig.showFps) {
            int fps = client.getCurrentFps();
            context.drawText(tr, Text.literal("FPS " + fps), x, y + line * 10, 0xFFFFFFFF, true);
            line++;
        }
        if (ModernConfig.showPing) {
            int ping = resolvePing(client);
            int color = ping < 80 ? 0xFF55FF55 : (ping < 150 ? 0xFFFFFF55 : 0xFFFF5555);
            context.drawText(tr, Text.literal("Ping " + ping + "ms"), x, y + line * 10, color, true);
            line++;
        }
        if (ModernConfig.showPerfBadge && PerformanceConfig.boostFps) {
            context.drawText(tr, Text.literal("Boost"), x, y + line * 10, 0xFF55FFAA, true);
            line++;
        }
        if (ModernConfig.showCps) {
            context.drawText(tr, Text.literal("CPS " + leftCps + " | " + rightCps), x, y + line * 10, 0xFFFFFFFF, true);
            line++;
        }
        if (ModernConfig.showCoords && client.player != null) {
            var p = client.player;
            String coords = String.format("XYZ %.0f %.0f %.0f", p.getX(), p.getY(), p.getZ());
            context.drawText(tr, Text.literal(coords), x, y + line * 10, 0xFFAAAAAA, true);
            line++;
        }
        if (ModernConfig.showArmor && client.player != null) {
            int armor = client.player.getArmor();
            context.drawText(tr, Text.literal("Armor " + armor), x, y + line * 10, 0xFF55AAFF, true);
            line++;
        }
        if (ModernConfig.showKeystrokes) {
            drawKeystrokes(context, tr, x, y + line * 10 + 4, client);
        }
    }

    private static int resolvePing(MinecraftClient client) {
        ClientPlayNetworkHandler handler = client.getNetworkHandler();
        if (handler == null || client.player == null) {
            return 0;
        }
        PlayerListEntry entry = handler.getPlayerListEntry(client.player.getUuid());
        return entry != null ? entry.getLatency() : 0;
    }

    private static void tickCps(MinecraftClient client) {
        long now = System.currentTimeMillis();
        if (now - cpsWindowStart >= 1000L) {
            leftCps = leftTicks;
            rightCps = rightTicks;
            leftTicks = 0;
            rightTicks = 0;
            cpsWindowStart = now;
        }
        if (client.options.attackKey.isPressed() && now - lastLeftClick > 40L) {
            lastLeftClick = now;
            leftTicks++;
        }
        if (client.options.useKey.isPressed() && now - lastRightClick > 40L) {
            lastRightClick = now;
            rightTicks++;
        }
    }

    private static void drawKeystrokes(DrawContext ctx, TextRenderer tr, int x, int y, MinecraftClient client) {
        boolean w = client.options.forwardKey.isPressed();
        boolean a = client.options.leftKey.isPressed();
        boolean s = client.options.backKey.isPressed();
        boolean d = client.options.rightKey.isPressed();
        boolean sp = client.options.sprintKey.isPressed();
        boolean spc = client.options.jumpKey.isPressed();
        boolean lmb = client.options.attackKey.isPressed();
        boolean rmb = client.options.useKey.isPressed();

        drawKey(ctx, tr, x + 18, y, "W", w);
        drawKey(ctx, tr, x, y + 12, "A", a);
        drawKey(ctx, tr, x + 18, y + 12, "S", s);
        drawKey(ctx, tr, x + 36, y + 12, "D", d);
        drawKey(ctx, tr, x + 56, y + 12, "Sp", sp);
        drawKey(ctx, tr, x + 76, y + 12, "_", spc);
        drawKey(ctx, tr, x, y + 28, "LMB " + leftCps, lmb);
        drawKey(ctx, tr, x + 44, y + 28, "RMB " + rightCps, rmb);
    }

    private static void drawKey(DrawContext ctx, TextRenderer tr, int x, int y, String label, boolean active) {
        int bg = active ? 0xAAFFFFFF : 0x55000000;
        int fg = active ? 0xFF111111 : 0xFFEEEEEE;
        int w = Math.max(16, tr.getWidth(label) + 6);
        ctx.fill(x, y, x + w, y + 10, bg);
        ctx.drawText(tr, Text.literal(label), x + 3, y + 1, fg, false);
    }
}
