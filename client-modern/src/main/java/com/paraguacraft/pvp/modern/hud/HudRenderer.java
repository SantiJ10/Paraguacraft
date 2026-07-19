package com.paraguacraft.pvp.modern.hud;

import com.paraguacraft.pvp.modern.ParaguacraftPvPModern;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.CombatStats;
import com.paraguacraft.pvp.modern.core.LauncherIpc;
import com.paraguacraft.pvp.modern.gui.GuiEditHudScreen;
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
    private static final int BASE_MUSIC_ART = 16;

    private HudRenderer() {}

    public static void register() {
        HudElementRegistry.addFirst(HUD_ID, HudRenderer::render);
    }

    /** Render en partida (Fabric HUD layer). */
    private static void render(DrawContext context, RenderTickCounter tickCounter) {
        renderAll(context, false, false);
    }

    /** Render desde modo edicion (preview real de mods activos). */
    public static void renderEditing(DrawContext context) {
        renderAll(context, true, true);
    }

    private static void renderAll(DrawContext context, boolean editing, boolean previewMusic) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client == null || client.player == null) {
            return;
        }
        if (!editing) {
            if (client.options.hudHidden || client.currentScreen != null) {
                return;
            }
        } else if (!(client.currentScreen instanceof GuiEditHudScreen)) {
            return;
        }

        if (ModernConfig.showFps) {
            drawLabeled(client.textRenderer, context, "FPS: ", String.valueOf(client.getCurrentFps()), ModernConfig.fpsX, ModernConfig.fpsY);
        }
        if (ModernConfig.showPing) {
            int ping = resolvePing(client);
            int color = ping < 80 ? 0xFF55FF55 : (ping < 150 ? 0xFFFFFF55 : 0xFFFF5555);
            drawLabeled(client.textRenderer, context, "Ping: ", ping + " ms", ModernConfig.pingX, ModernConfig.pingY, color);
        }
        if (ModernConfig.showCps) {
            drawLabeled(client.textRenderer, context, "CPS: ", String.valueOf(HudCpsTracker.leftCps()), ModernConfig.cpsX, ModernConfig.cpsY);
        }
        if (ModernConfig.showCoords) {
            var p = client.player;
            String coords = String.format("X: %.0f  Y: %.0f  Z: %.0f", p.getX(), p.getY(), p.getZ());
            context.drawText(client.textRenderer, Text.literal(coords), ModernConfig.coordsX, ModernConfig.coordsY, 0xFF00E5FF, true);
        }
        if (ModernConfig.showKeystrokes) {
            drawKeystrokes(context, client.textRenderer, client);
        }
        if (ModernConfig.showBlockCount) {
            drawBlockCount(context, client.textRenderer, client);
        }
        if (ModernConfig.showArmor) {
            drawArmor(context, client);
        }
        if (ModernConfig.showHeldItem) {
            drawHeldItem(context, client);
        }
        if (ModernConfig.showBedwarsResources) {
            drawBedwarsResources(context, client.textRenderer, client);
        }
        if (ModernConfig.showMusicHud) {
            drawMusicOverlay(context, client.textRenderer, client, previewMusic);
        }
        if (ModernConfig.comboCounter) {
            drawLabeled(client.textRenderer, context, "Combo: ", String.valueOf(CombatStats.comboCount), ModernConfig.comboX, ModernConfig.comboY, 0xFF00E5FF);
        }
        if (ModernConfig.showPotions) {
            drawPotions(context, client);
        }
        if (ModernConfig.showCompass) {
            drawCompass(context, client.textRenderer, client);
        }
        if (ModernConfig.showTntCountdown) {
            drawNearestTnt(context, client.textRenderer, client);
        }
    }

    public static int musicPanelWidth() {
        return musicPanelWidth(LauncherIpc.get(), true);
    }

    public static int musicPanelWidth(LauncherIpc.Snapshot snap, boolean preview) {
        if (!ModernConfig.showMusicHud) {
            return scaledMusic(80);
        }
        boolean playing = preview || (snap != null && snap.musicPlaying);
        if (!playing) {
            return scaledMusic(80);
        }
        MinecraftClient client = MinecraftClient.getInstance();
        if (client == null) {
            return scaledMusic(80);
        }
        TextRenderer tr = client.textRenderer;
        String title = preview ? "Vista previa" : snap.musicTitle;
        String artist = preview ? "Spotify / YouTube" : snap.musicArtist;
        if (title == null || title.isEmpty()) {
            title = "Reproduciendo";
        }
        if (artist == null || artist.isEmpty()) {
            artist = "Spotify / YouTube";
        }
        title = clampMusicText(title, 22);
        artist = clampMusicText(artist, 28);

        int pad = scaledMusic(6);
        int textW = Math.max(tr.getWidth(title), tr.getWidth(artist)) + pad * 2;
        String imageUrl = preview ? "" : (snap != null ? snap.musicImageUrl : "");
        boolean showArt = ModernConfig.showMusicAlbumArt
            && (preview || (imageUrl != null && !imageUrl.isEmpty()));
        if (showArt) {
            textW += musicArtSize() + scaledMusic(4);
        }
        return Math.max(scaledMusic(72), textW);
    }

    public static int musicPanelHeight(LauncherIpc.Snapshot snap, boolean preview) {
        int art = musicArtSize();
        int baseH = scaledMusic(28);
        if (ModernConfig.showMusicAlbumArt && (preview || (snap != null && !snap.musicImageUrl.isEmpty()))) {
            return Math.max(baseH, scaledMusic(8) + art + scaledMusic(4));
        }
        return baseH;
    }

    public static int armorPanelHeight(MinecraftClient client) {
        if (client == null || client.player == null) {
            return 64;
        }
        EquipmentSlot[] slots = {EquipmentSlot.HEAD, EquipmentSlot.CHEST, EquipmentSlot.LEGS, EquipmentSlot.FEET};
        int count = 0;
        for (EquipmentSlot slot : slots) {
            if (!client.player.getEquippedStack(slot).isEmpty()) {
                count++;
            }
        }
        return Math.max(16, count * 16);
    }

    public static int potionPanelHeight(MinecraftClient client) {
        if (client == null || client.player == null || client.player.getStatusEffects().isEmpty()) {
            return 24;
        }
        return client.player.getStatusEffects().size() * 24;
    }

    private static int scaledMusic(int value) {
        return Math.max(1, value * ModernConfig.musicHudScale / 100);
    }

    private static int musicArtSize() {
        return Math.max(12, scaledMusic(BASE_MUSIC_ART));
    }

    private static void drawLabeled(TextRenderer tr, DrawContext ctx, String label, String value, int x, int y) {
        drawLabeled(tr, ctx, label, value, x, y, 0xFFFFFFFF);
    }

    private static void drawLabeled(TextRenderer tr, DrawContext ctx, String label, String value, int x, int y, int valueColor) {
        ctx.drawText(tr, Text.literal(label), x, y, 0xFFFFFFFF, true);
        ctx.drawText(tr, Text.literal(value), x + tr.getWidth(label), y, valueColor, true);
    }

    private static void drawKeystrokes(DrawContext ctx, TextRenderer tr, MinecraftClient client) {
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

    /** Columna vertical casco → botas, estilo 1.8.9 (icono + brillo encantamiento). */
    private static void drawArmor(DrawContext ctx, MinecraftClient client) {
        EquipmentSlot[] slots = {EquipmentSlot.HEAD, EquipmentSlot.CHEST, EquipmentSlot.LEGS, EquipmentSlot.FEET};
        int yOffset = 0;
        TextRenderer tr = client.textRenderer;
        for (EquipmentSlot slot : slots) {
            ItemStack stack = client.player.getEquippedStack(slot);
            if (stack.isEmpty()) {
                continue;
            }
            int x = ModernConfig.armorX;
            int y = ModernConfig.armorY + yOffset;
            int iconX = x + 25;
            ctx.drawItem(client.player, stack, iconX, y, 0);
            if (ModernConfig.showArmorPercentage && stack.isDamageable()) {
                int max = stack.getMaxDamage();
                int dmg = stack.getDamage();
                int percent = max > 0 ? (int) (((max - dmg) * 100.0F) / max) : 100;
                String text = percent + "%";
                int color = percent < 25 ? 0xFFFF5555 : (percent < 50 ? 0xFFFFCC55 : 0xFF55FF55);
                ctx.drawText(tr, Text.literal(text), x + 22 - tr.getWidth(text), y + 4, color, true);
            }
            yOffset += 16;
        }
    }

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
            String line = Enchantment.getName(ench, level).getString();
            if (!line.isEmpty()) {
                ctx.drawText(client.textRenderer, Text.literal(line), textX, textY, UiTheme.textDim(), true);
                textY += 10;
            }
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

    /** Brújula detallada con recorte, estilo 1.8.9. */
    private static void drawCompass(DrawContext ctx, TextRenderer tr, MinecraftClient client) {
        int centerX = client.getWindow().getScaledWidth() / 2;
        int y = ModernConfig.compassY;
        int boxW = 220;
        int boxH = 16;
        int x = centerX - boxW / 2;

        ctx.enableScissor(x, y, x + boxW, y + boxH);
        float yaw = (client.player.getYaw() % 360.0F + 360.0F) % 360.0F;
        for (int i = (int) yaw - 60; i < (int) yaw + 60; i++) {
            int angle = (i + 360) % 360;
            if (angle % 15 != 0) {
                continue;
            }
            float offset = i - yaw;
            float px = centerX + offset * 2.0F;
            String markerText = compassMarker(angle);
            int color = compassColor(angle);
            ctx.drawText(tr, Text.literal(markerText), (int) px - tr.getWidth(markerText) / 2, y + 4, color, true);
        }
        ctx.disableScissor();
        ctx.fill(centerX - 1, y, centerX + 1, y + 4, 0xFF00E5FF);
    }

    private static String compassMarker(int angle) {
        return switch (angle) {
            case 0, 360 -> "S";
            case 45 -> "SO";
            case 90 -> "O";
            case 135 -> "NO";
            case 180 -> "N";
            case 225 -> "NE";
            case 270 -> "E";
            case 315 -> "SE";
            default -> String.valueOf(angle);
        };
    }

    private static int compassColor(int angle) {
        if (angle == 0 || angle == 90 || angle == 180 || angle == 270 || angle == 360) {
            return 0xFF00E5FF;
        }
        if (angle % 45 == 0) {
            return 0xFFFFFFFF;
        }
        return 0xFFAAAAAA;
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

    private static void drawMusicOverlay(DrawContext ctx, TextRenderer tr, MinecraftClient client, boolean preview) {
        LauncherIpc.Snapshot snap = LauncherIpc.get();
        boolean playing = snap.musicPlaying;
        if (!playing && !preview) {
            return;
        }

        String title = playing ? snap.musicTitle : "Vista previa";
        String artist = playing ? snap.musicArtist : "Spotify / YouTube";
        String imageUrl = playing ? snap.musicImageUrl : "";

        int x = ModernConfig.overlayHudX;
        int y = ModernConfig.overlayHudY;
        int w = musicPanelWidth(snap, preview);
        int alpha = Math.max(0, Math.min(255, ModernConfig.musicHudAlpha));
        int panelH = musicPanelHeight(snap, preview);
        if (alpha > 0) {
            int bg = (alpha << 24) | 0x0A0C14;
            ctx.fill(x, y, x + w, y + panelH, bg);
        }
        ctx.fill(x, y, x + w, y + 1, 0x4400E5FF);

        int pad = scaledMusic(6);
        int artSize = musicArtSize();
        int textX = x + pad;
        int textY = y + pad;
        boolean wantsArt = ModernConfig.showMusicAlbumArt
            && (preview || (imageUrl != null && !imageUrl.isEmpty()));
        if (wantsArt) {
            int artX = x + pad;
            int artY = y + pad;
            Identifier art = preview ? null : MusicArtCache.get(imageUrl);
            if (art != null) {
                ctx.drawTexture(
                    RenderPipelines.GUI_TEXTURED,
                    art,
                    artX,
                    artY,
                    0f,
                    0f,
                    artSize,
                    artSize,
                    MusicArtCache.getTexWidth(),
                    MusicArtCache.getTexHeight()
                );
            } else if (preview) {
                ctx.fill(artX, artY, artX + artSize, artY + artSize, 0xFF202030);
                ctx.drawText(tr, Text.literal("♪"), artX + artSize / 2 - 3, artY + artSize / 2 - 4, UiTheme.accent(), false);
            } else {
                ctx.drawItem(new ItemStack(Items.MUSIC_DISC_13), artX, artY);
            }
            textX = artX + artSize + scaledMusic(4);
        }

        if (title == null || title.isEmpty()) {
            title = "Reproduciendo";
        }
        if (artist == null || artist.isEmpty()) {
            artist = "Spotify / YouTube";
        }
        int innerMax = (x + w) - textX - pad;
        title = clampToWidth(title, innerMax, tr);
        artist = clampToWidth(artist, innerMax, tr);
        ctx.drawText(tr, Text.literal(title), textX, textY, UiTheme.accent(), true);
        ctx.drawText(tr, Text.literal(artist), textX, textY + scaledMusic(11), UiTheme.textDim(), true);
    }

    private static String clampMusicText(String text, int maxChars) {
        if (text == null) {
            return "";
        }
        if (text.length() <= maxChars) {
            return text;
        }
        return text.substring(0, Math.max(1, maxChars - 1)) + "…";
    }

    private static String clampToWidth(String text, int maxPx, TextRenderer tr) {
        if (text == null || text.isEmpty() || maxPx <= 0) {
            return text == null ? "" : text;
        }
        if (tr.getWidth(text) <= maxPx) {
            return text;
        }
        String ell = "…";
        int end = text.length();
        while (end > 1 && tr.getWidth(text.substring(0, end) + ell) > maxPx) {
            end--;
        }
        return text.substring(0, end) + ell;
    }

    private static int resolvePing(MinecraftClient client) {
        ClientPlayNetworkHandler handler = client.getNetworkHandler();
        if (handler == null || client.player == null) {
            return 0;
        }
        PlayerListEntry self = handler.getPlayerListEntry(client.player.getUuid());
        if (self != null) {
            return Math.max(0, self.getLatency());
        }
        int sum = 0;
        int count = 0;
        for (PlayerListEntry entry : handler.getPlayerList()) {
            int lat = entry.getLatency();
            if (lat > 0) {
                sum += lat;
                count++;
            }
        }
        if (count > 0) {
            return sum / count;
        }
        return 0;
    }
}
