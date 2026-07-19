package com.paraguacraft.pvp.modern.hud;

import com.paraguacraft.pvp.modern.ParaguacraftPvPModern;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.CombatStats;
import com.paraguacraft.pvp.modern.core.LauncherIpc;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.fabricmc.fabric.api.client.rendering.v1.hud.HudElementRegistry;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.font.TextRenderer;
import net.minecraft.client.gl.RenderPipelines;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.hud.InGameHud;
import net.minecraft.client.network.ClientPlayNetworkHandler;
import net.minecraft.client.network.PlayerListEntry;
import net.minecraft.client.render.RenderTickCounter;
import net.minecraft.component.type.ItemEnchantmentsComponent;
import net.minecraft.enchantment.Enchantment;
import net.minecraft.enchantment.EnchantmentHelper;
import net.minecraft.entity.EquipmentSlot;
import net.minecraft.entity.TntEntity;
import net.minecraft.entity.effect.StatusEffectInstance;
import net.minecraft.entity.effect.StatusEffectUtil;
import net.minecraft.item.BlockItem;
import net.minecraft.item.ItemStack;
import net.minecraft.item.Items;
import net.minecraft.registry.entry.RegistryEntry;
import net.minecraft.text.Text;
import net.minecraft.util.Identifier;
import org.lwjgl.glfw.GLFW;

/** HUD estilo Paraguacraft 1.8.9: FPS, ping, CPS, keystrokes, armadura, BW y musica. */
public final class HudRenderer {

    private static final Identifier HUD_ID = Identifier.of(ParaguacraftPvPModern.MOD_ID, "hud");
    private static final int MUSIC_ART = 16;

    private HudRenderer() {}

    public static void register() {
        HudElementRegistry.addFirst(HUD_ID, HudRenderer::render);
    }

    private static void render(DrawContext context, RenderTickCounter tickCounter) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client == null || client.options.hudHidden || client.currentScreen != null || client.player == null) {
            return;
        }
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
            drawLabeled(tr, context, "CPS: ", String.valueOf(HudCpsTracker.leftCps()), ModernConfig.cpsX, ModernConfig.cpsY);
        }
        if (ModernConfig.showCoords) {
            var p = client.player;
            String coords = String.format("X: %.0f  Y: %.0f  Z: %.0f", p.getX(), p.getY(), p.getZ());
            context.drawText(tr, Text.literal(coords), ModernConfig.hudX, coordsY(), 0xFF00E5FF, true);
        }
        if (ModernConfig.showKeystrokes) {
            drawKeystrokes(context, tr);
        }
        if (ModernConfig.showBlockCount) {
            drawBlockCount(context, tr, client);
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
            drawMusicOverlay(context, tr, client);
        }
        if (ModernConfig.comboCounter) {
            drawLabeled(tr, context, "Combo: ", String.valueOf(CombatStats.comboCount), ModernConfig.comboX, ModernConfig.comboY, 0xFF00E5FF);
        }
        if (ModernConfig.showPotions) {
            drawPotions(context, client);
        }
        if (ModernConfig.showCompass) {
            drawCompass(context, tr, client);
        }
        if (ModernConfig.showTntCountdown) {
            drawNearestTnt(context, tr, client);
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

    /** Columna vertical de iconos (casco → botas), estilo 1.8.9. */
    private static void drawArmor(DrawContext ctx, MinecraftClient client) {
        EquipmentSlot[] slots = {EquipmentSlot.HEAD, EquipmentSlot.CHEST, EquipmentSlot.LEGS, EquipmentSlot.FEET};
        int yOffset = 0;
        for (EquipmentSlot slot : slots) {
            ItemStack stack = client.player.getEquippedStack(slot);
            if (stack.isEmpty()) {
                continue;
            }
            int y = ModernConfig.armorY + yOffset;
            ctx.drawItem(stack, ModernConfig.armorX, y);
            if (ModernConfig.showArmorPercentage && stack.isDamageable()) {
                int max = stack.getMaxDamage();
                int dmg = stack.getDamage();
                int percent = max > 0 ? (int) (((max - dmg) * 100.0F) / max) : 100;
                String text = percent + "%";
                int color = percent < 25 ? 0xFFFF5555 : (percent < 50 ? 0xFFFFCC55 : 0xFF55FF55);
                ctx.drawText(client.textRenderer, Text.literal(text), ModernConfig.armorX + 18, y + 4, color, true);
            }
            yOffset += 16;
        }
    }

    /** Bloques colocables en inventario (BedWars, SkyWars, etc.). */
    private static void drawBlockCount(DrawContext ctx, TextRenderer tr, MinecraftClient client) {
        int blocks = countPlaceableBlocks(client);
        if (blocks <= 0) {
            return;
        }
        int x = ModernConfig.blocksX;
        int y = ModernConfig.blocksY;
        ctx.drawItem(new ItemStack(Items.WHITE_WOOL), x, y);
        ctx.drawText(tr, Text.literal(String.valueOf(blocks)), x + 18, y + 4, 0xFFFFFFFF, true);
    }

    private static int countPlaceableBlocks(MinecraftClient client) {
        int total = 0;
        for (int i = 0; i < client.player.getInventory().size(); i++) {
            ItemStack stack = client.player.getInventory().getStack(i);
            if (stack.isEmpty()) {
                continue;
            }
            if (stack.getItem() instanceof BlockItem) {
                total += stack.getCount();
            }
        }
        return total;
    }

    /** Nombre + encantamientos, estilo 1.8.9. */
    private static void drawHeldItem(DrawContext ctx, MinecraftClient client) {
        ItemStack stack = client.player.getMainHandStack();
        if (stack.isEmpty()) {
            return;
        }
        int x = ModernConfig.heldX;
        int y = ModernConfig.heldY;
        ctx.drawItem(stack, x, y);
        int textX = x + 20;
        int textY = y + 1;
        String displayName = stack.getName().getString();
        int nameColor = EnchantmentHelper.hasEnchantments(stack) ? UiTheme.accent() : UiTheme.TEXT;
        ctx.drawText(client.textRenderer, Text.literal(displayName), textX, textY, nameColor, true);
        textY += 10;
        ItemEnchantmentsComponent enchants = EnchantmentHelper.getEnchantments(stack);
        for (var entry : enchants.getEnchantmentEntries()) {
            RegistryEntry<Enchantment> ench = entry.getKey();
            int level = entry.getIntValue();
            String line = formatEnchantLine(ench, level);
            if (!line.isEmpty()) {
                ctx.drawText(client.textRenderer, Text.literal(line), textX, textY, UiTheme.textDim(), true);
                textY += 10;
            }
        }
    }

    private static String formatEnchantLine(RegistryEntry<Enchantment> ench, int level) {
        Text name = Enchantment.getName(ench, level);
        return name.getString();
    }

    /** Recursos BW: columna vertical transparente con icono + cantidad. */
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
        int rowH = 16;
        if (!ModernConfig.bwResTransparentBg) {
            ctx.fill(x - 2, y - 2, x + 42, y + rowH * 4 + 2, 0x88000000);
        }
        drawBwRow(ctx, tr, new ItemStack(Items.IRON_INGOT), iron, x, y);
        drawBwRow(ctx, tr, new ItemStack(Items.GOLD_INGOT), gold, x, y + rowH);
        drawBwRow(ctx, tr, new ItemStack(Items.DIAMOND), diamond, x, y + rowH * 2);
        drawBwRow(ctx, tr, new ItemStack(Items.EMERALD), emerald, x, y + rowH * 3);
    }

    private static void drawBwRow(DrawContext ctx, TextRenderer tr, ItemStack icon, int count, int x, int y) {
        ctx.drawItem(icon, x, y);
        ctx.drawText(tr, Text.literal(String.valueOf(count)), x + 18, y + 4, 0xFFFFFFFF, true);
    }

    /** Pociones: icono + nombre + duracion formateada (estilo 1.8.9). */
    private static void drawPotions(DrawContext ctx, MinecraftClient client) {
        int y = ModernConfig.potionY;
        for (StatusEffectInstance effect : client.player.getStatusEffects()) {
            var type = effect.getEffectType();
            Identifier tex = InGameHud.getEffectTexture(type);
            ctx.drawGuiTexture(RenderPipelines.GUI_TEXTURED, tex, ModernConfig.potionX, y, 18, 18);
            String name = Text.translatable(type.value().getTranslationKey()).getString();
            if (effect.getAmplifier() > 0) {
                name += " " + (effect.getAmplifier() + 1);
            }
            ctx.drawText(client.textRenderer, Text.literal(name), ModernConfig.potionX + 22, y + 1, UiTheme.TEXT, true);
            if (effect.getDuration() > 0) {
                String duration = StatusEffectUtil.getDurationText(effect, 1.0F, client.world.getTickManager().getTickRate()).getString();
                ctx.drawText(client.textRenderer, Text.literal(duration), ModernConfig.potionX + 22, y + 11, UiTheme.textDim(), true);
            }
            y += 24;
        }
    }

    private static void drawCompass(DrawContext ctx, TextRenderer tr, MinecraftClient client) {
        float yaw = (client.player.getYaw() % 360 + 360) % 360;
        String dir = compassDirection(yaw);
        int cx = client.getWindow().getScaledWidth() / 2;
        ctx.drawCenteredTextWithShadow(tr, Text.literal(dir), cx, ModernConfig.compassY, 0xFF00E5FF);
    }

    private static String compassDirection(float yaw) {
        if (yaw >= 315 || yaw < 45) return "S";
        if (yaw < 135) return "W";
        if (yaw < 225) return "N";
        return "E";
    }

    private static void drawNearestTnt(DrawContext ctx, TextRenderer tr, MinecraftClient client) {
        if (client.world == null) {
            return;
        }
        TntEntity nearest = null;
        double best = 256;
        for (var entity : client.world.getEntities()) {
            if (!(entity instanceof TntEntity tnt) || tnt.getFuse() <= 0) {
                continue;
            }
            double d = client.player.squaredDistanceTo(entity);
            if (d < best) {
                best = d;
                nearest = tnt;
            }
        }
        if (nearest == null) {
            return;
        }
        String label = String.format("TNT: %.1fs", nearest.getFuse() / 20.0f);
        ctx.drawText(tr, Text.literal(label), ModernConfig.hudX, ModernConfig.hudY + 52, 0xFFFF5555, true);
    }

    private static void drawMusicOverlay(DrawContext ctx, TextRenderer tr, MinecraftClient client) {
        LauncherIpc.Snapshot snap = LauncherIpc.get();
        if (!snap.musicPlaying) {
            return;
        }
        if (ModernConfig.showMusicAlbumArt && !snap.musicImageUrl.isEmpty()) {
            MusicArtCache.request(snap.musicImageUrl);
        }
        int x = ModernConfig.overlayHudX;
        int y = ModernConfig.overlayHudY;
        int w = ModernConfig.overlayHudW;
        int alpha = Math.max(0, Math.min(255, ModernConfig.musicHudAlpha));
        int bg = (alpha << 24) | 0x101018;
        int panelH = 28;
        if (ModernConfig.showMusicAlbumArt && !snap.musicImageUrl.isEmpty()) {
            panelH = Math.max(panelH, 8 + MUSIC_ART + 4);
        }
        ctx.fill(x, y, x + w, y + panelH, bg);
        int textX = x + 4;
        int textY = y + 4;
        if (ModernConfig.showMusicAlbumArt && !snap.musicImageUrl.isEmpty()) {
            Identifier art = MusicArtCache.get(snap.musicImageUrl);
            if (art != null) {
                ctx.drawTexture(RenderPipelines.GUI_TEXTURED, art, x + 4, y + 4, 0f, 0f, MUSIC_ART, MUSIC_ART, MUSIC_ART, MUSIC_ART);
                textX = x + 4 + MUSIC_ART + 4;
            }
        }
        String title = snap.musicTitle.isEmpty() ? "Reproduciendo" : snap.musicTitle;
        String artist = snap.musicArtist.isEmpty() ? "Spotify / YouTube" : snap.musicArtist;
        if (title.length() > 24) {
            title = title.substring(0, 22) + "…";
        }
        if (artist.length() > 28) {
            artist = artist.substring(0, 26) + "…";
        }
        ctx.drawText(tr, Text.literal(title), textX, textY, UiTheme.TEXT, true);
        ctx.drawText(tr, Text.literal(artist), textX, textY + 12, UiTheme.textDim(), true);
    }

    private static int resolvePing(MinecraftClient client) {
        ClientPlayNetworkHandler handler = client.getNetworkHandler();
        if (handler == null || client.player == null) {
            return 0;
        }
        PlayerListEntry entry = handler.getPlayerListEntry(client.player.getUuid());
        return entry != null ? Math.max(0, entry.getLatency()) : 0;
    }
}
