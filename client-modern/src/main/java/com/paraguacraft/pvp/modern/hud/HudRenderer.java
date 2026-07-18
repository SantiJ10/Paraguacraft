package com.paraguacraft.pvp.modern.hud;

import com.paraguacraft.pvp.modern.ParaguacraftPvPModern;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.LauncherIpc;
import com.paraguacraft.pvp.modern.core.PerformanceConfig;
import net.fabricmc.fabric.api.client.rendering.v1.hud.HudElementRegistry;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.font.TextRenderer;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.network.ClientPlayNetworkHandler;
import net.minecraft.client.network.PlayerListEntry;
import net.minecraft.client.render.RenderTickCounter;
import net.minecraft.enchantment.EnchantmentHelper;
import net.minecraft.entity.EquipmentSlot;
import net.minecraft.item.ItemStack;
import net.minecraft.item.Items;
import net.minecraft.text.Text;
import net.minecraft.util.Identifier;
import org.lwjgl.glfw.GLFW;

/** HUD estilo Paraguacraft 1.8.9: FPS, ping, CPS, keystrokes, armadura, BW y musica. */
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
        if (client == null || client.options.hudHidden || client.currentScreen != null || client.player == null) {
            return;
        }
        tickCps(client);
        TextRenderer tr = client.textRenderer;

        if (ModernConfig.showFps) {
            drawLabeled(tr, context, "FPS: ", String.valueOf(client.getCurrentFps()), ModernConfig.fpsX, ModernConfig.fpsY);
        }
        if (ModernConfig.showPing) {
            int ping = resolvePing(client);
            int color = ping < 80 ? 0xFF55FF55 : (ping < 150 ? 0xFFFFFF55 : 0xFFFF5555);
            drawLabeled(tr, context, "Ping: ", ping + " ms", ModernConfig.pingX, ModernConfig.pingY, color);
        }
        if (ModernConfig.showCps) {
            drawLabeled(tr, context, "CPS: ", String.valueOf(leftCps), ModernConfig.cpsX, ModernConfig.cpsY);
        }
        if (ModernConfig.showCoords) {
            var p = client.player;
            String coords = String.format("X: %.0f  Y: %.0f  Z: %.0f", p.getX(), p.getY(), p.getZ());
            context.drawText(tr, Text.literal(coords), ModernConfig.hudX, coordsY(), 0xFF00E5FF, true);
        }
        if (ModernConfig.showPerfBadge && PerformanceConfig.boostFps) {
            context.drawText(tr, Text.literal("Boost FPS"), ModernConfig.hudX, ModernConfig.hudY, 0xFF55FFAA, true);
        }
        if (ModernConfig.showKeystrokes) {
            drawKeystrokes(context, tr);
        }
        if (ModernConfig.showArmor) {
            drawArmor(context, client);
        }
        if (ModernConfig.showHeldItem) {
            drawHeldItem(context, client);
        }
        if (ModernConfig.showBedwarsResources) {
            drawBedwarsResources(context, tr, client);
        }
        if (ModernConfig.showMusicHud) {
            drawMusicOverlay(context, tr);
        }
    }

    private static int coordsY() {
        return ModernConfig.hudY + 40;
    }

    private static void drawLabeled(TextRenderer tr, DrawContext ctx, String label, String value, int x, int y) {
        drawLabeled(tr, ctx, label, value, x, y, 0xFFFFFFFF);
    }

    private static void drawLabeled(TextRenderer tr, DrawContext ctx, String label, String value, int x, int y, int valueColor) {
        ctx.drawText(tr, Text.literal(label), x, y, 0xFFFFFFFF, true);
        ctx.drawText(tr, Text.literal(value), x + tr.getWidth(label), y, valueColor, true);
    }

    /** Keystrokes 1.8.9: cajas 20x20, WASD + LMB/RMB. */
    private static void drawKeystrokes(DrawContext ctx, TextRenderer tr) {
        MinecraftClient client = MinecraftClient.getInstance();
        int x = ModernConfig.keysX;
        int y = ModernConfig.keysY;
        int gap = 2;
        int size = 20;

        drawKeyBox(ctx, tr, x + size + gap, y, size, size, client.options.forwardKey);
        drawKeyBox(ctx, tr, x, y + size + gap, size, size, client.options.leftKey);
        drawKeyBox(ctx, tr, x + size + gap, y + size + gap, size, size, client.options.backKey);
        drawKeyBox(ctx, tr, x + (size + gap) * 2, y + size + gap, size, size, client.options.rightKey);
        drawMouseBox(ctx, tr, x, y + (size + gap) * 2, 31, size, GLFW.GLFW_MOUSE_BUTTON_LEFT, "LMB");
        drawMouseBox(ctx, tr, x + 31 + gap, y + (size + gap) * 2, 31, size, GLFW.GLFW_MOUSE_BUTTON_RIGHT, "RMB");
    }

    private static void drawKeyBox(DrawContext ctx, TextRenderer tr, int x, int y, int w, int h, net.minecraft.client.option.KeyBinding key) {
        boolean pressed = key.isPressed();
        int bg = pressed ? 0x88FFFFFF : 0x88000000;
        int fg = pressed ? 0xFF111111 : 0xFFFFFFFF;
        ctx.fill(x, y, x + w, y + h, bg);
        String label = key.getBoundKeyLocalizedText().getString();
        if (label.length() > 3) {
            label = label.substring(0, 1);
        }
        ctx.drawText(tr, Text.literal(label), x + w / 2 - tr.getWidth(label) / 2, y + h / 2 - 4, fg, false);
    }

    private static void drawMouseBox(DrawContext ctx, TextRenderer tr, int x, int y, int w, int h, int button, String label) {
        boolean pressed = GLFW.glfwGetMouseButton(MinecraftClient.getInstance().getWindow().getHandle(), button) == GLFW.GLFW_PRESS;
        int bg = pressed ? 0x88FFFFFF : 0x88000000;
        int fg = pressed ? 0xFF111111 : 0xFFFFFFFF;
        ctx.fill(x, y, x + w, y + h, bg);
        ctx.drawText(tr, Text.literal(label), x + w / 2 - tr.getWidth(label) / 2, y + h / 2 - 4, fg, false);
    }

    private static void drawArmor(DrawContext ctx, MinecraftClient client) {
        EquipmentSlot[] slots = {EquipmentSlot.HEAD, EquipmentSlot.CHEST, EquipmentSlot.LEGS, EquipmentSlot.FEET};
        int yOffset = 0;
        for (EquipmentSlot slot : slots) {
            ItemStack stack = client.player.getEquippedStack(slot);
            if (stack.isEmpty()) {
                continue;
            }
            int y = ModernConfig.armorY + yOffset;
            ctx.drawItem(stack, ModernConfig.armorX + 25, y);
            if (ModernConfig.showArmorPercentage && stack.isDamageable()) {
                int max = stack.getMaxDamage();
                int dmg = stack.getDamage();
                int percent = max > 0 ? (int) (((max - dmg) * 100.0F) / max) : 100;
                String text = percent + "%";
                int color = percent < 25 ? 0xFFFF5555 : (percent < 50 ? 0xFFFFCC55 : 0xFF55FF55);
                ctx.drawText(client.textRenderer, Text.literal(text), ModernConfig.armorX, y + 4, color, true);
            }
            yOffset += 16;
        }
    }

    private static void drawHeldItem(DrawContext ctx, MinecraftClient client) {
        ItemStack stack = client.player.getMainHandStack();
        if (stack.isEmpty()) {
            return;
        }
        int x = ModernConfig.heldX;
        int y = ModernConfig.heldY;
        ctx.drawItem(stack, x, y);
        ctx.drawStackOverlay(client.textRenderer, stack, x, y);
        if (EnchantmentHelper.hasEnchantments(stack)) {
            ctx.drawText(client.textRenderer, Text.literal("Ench"), x + 20, y + 4, 0xFF00E5FF, true);
        }
    }

    private static void drawBedwarsResources(DrawContext ctx, TextRenderer tr, MinecraftClient client) {
        int iron = 0;
        int gold = 0;
        int diamond = 0;
        int emerald = 0;
        for (int i = 0; i < client.player.getInventory().size(); i++) {
            ItemStack stack = client.player.getInventory().getStack(i);
            if (stack.isEmpty()) {
                continue;
            }
            if (stack.isOf(Items.IRON_INGOT)) {
                iron += stack.getCount();
            } else if (stack.isOf(Items.GOLD_INGOT)) {
                gold += stack.getCount();
            } else if (stack.isOf(Items.DIAMOND)) {
                diamond += stack.getCount();
            } else if (stack.isOf(Items.EMERALD)) {
                emerald += stack.getCount();
            }
        }
        int x = ModernConfig.bwResX;
        int y = ModernConfig.bwResY;
        ctx.fill(x - 2, y - 2, x + 92, y + 34, 0x88000000);
        ctx.drawItem(new ItemStack(Items.IRON_INGOT), x, y);
        ctx.drawText(tr, Text.literal(String.valueOf(iron)), x + 18, y + 4, 0xFFFFFFFF, true);
        ctx.drawItem(new ItemStack(Items.GOLD_INGOT), x, y + 12);
        ctx.drawText(tr, Text.literal(String.valueOf(gold)), x + 18, y + 16, 0xFFFFFFFF, true);
        ctx.drawItem(new ItemStack(Items.DIAMOND), x + 44, y);
        ctx.drawText(tr, Text.literal(String.valueOf(diamond)), x + 62, y + 4, 0xFFFFFFFF, true);
        ctx.drawItem(new ItemStack(Items.EMERALD), x + 44, y + 12);
        ctx.drawText(tr, Text.literal(String.valueOf(emerald)), x + 62, y + 16, 0xFFFFFFFF, true);
    }

    private static void drawMusicOverlay(DrawContext ctx, TextRenderer tr) {
        LauncherIpc.Snapshot snap = LauncherIpc.get();
        if (!snap.valid || !snap.musicPlaying) {
            return;
        }
        int x = ModernConfig.overlayHudX;
        int y = ModernConfig.overlayHudY;
        int w = ModernConfig.overlayHudW;
        int alpha = Math.max(0, Math.min(255, ModernConfig.musicHudAlpha));
        int bg = (alpha << 24);
        ctx.fill(x, y, x + w, y + 28, bg | 0x101018);
        String title = snap.musicTitle.isEmpty() ? "Reproduciendo" : snap.musicTitle;
        String artist = snap.musicArtist.isEmpty() ? "Spotify / YouTube" : snap.musicArtist;
        if (title.length() > 24) {
            title = title.substring(0, 22) + "…";
        }
        if (artist.length() > 28) {
            artist = artist.substring(0, 26) + "…";
        }
        ctx.drawText(tr, Text.literal(title), x + 4, y + 4, 0xFFFFFFFF, true);
        ctx.drawText(tr, Text.literal(artist), x + 4, y + 16, 0xFFAAAAAA, true);
    }

    private static int resolvePing(MinecraftClient client) {
        ClientPlayNetworkHandler handler = client.getNetworkHandler();
        if (handler == null || client.player == null) {
            return 0;
        }
        PlayerListEntry entry = handler.getPlayerListEntry(client.player.getUuid());
        return entry != null ? Math.max(0, entry.getLatency()) : 0;
    }

    private static void tickCps(MinecraftClient client) {
        long now = System.currentTimeMillis();
        if (now - cpsWindowStart > 1000L) {
            leftCps = leftTicks;
            rightCps = rightTicks;
            leftTicks = 0;
            rightTicks = 0;
            cpsWindowStart = now;
        }
        long window = client.getWindow().getHandle();
        boolean left = GLFW.glfwGetMouseButton(window, GLFW.GLFW_MOUSE_BUTTON_LEFT) == GLFW.GLFW_PRESS;
        boolean right = GLFW.glfwGetMouseButton(window, GLFW.GLFW_MOUSE_BUTTON_RIGHT) == GLFW.GLFW_PRESS;
        if (left && now - lastLeftClick > 50L) {
            lastLeftClick = now;
            leftTicks++;
        }
        if (right && now - lastRightClick > 50L) {
            lastRightClick = now;
            rightTicks++;
        }
    }
}
