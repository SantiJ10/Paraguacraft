package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.item.ItemStack;
import net.minecraft.item.Items;
import net.minecraft.text.Text;

import java.util.Locale;

/** Icono unico por mod en el menu interno (sin logo Paraguacraft en cada tarjeta). */
public final class ModCardIcons {

    private ModCardIcons() {}

    public static void draw(DrawContext ctx, String label, int x, int y, int size) {
        ItemStack stack = iconFor(label);
        int bg = accentFor(label);
        ctx.fill(x, y, x + size, y + size, 0xFF000000 | (bg & 0x00FFFFFF));
        ctx.fill(x, y, x + size, y + 1, UiTheme.accent());
        if (!stack.isEmpty()) {
            ctx.drawItem(stack, x + (size - 16) / 2, y + (size - 16) / 2);
        } else {
            var tr = MinecraftClient.getInstance().textRenderer;
            String mark = markFor(label);
            ctx.drawText(tr, Text.literal(mark), x + size / 2 - tr.getWidth(mark) / 2, y + size / 2 - 4, 0xFFFFFFFF, true);
        }
    }

    private static ItemStack iconFor(String label) {
        String key = label.toLowerCase(Locale.ROOT);
        if (key.contains("fps") || key.contains("preset")) {
            return new ItemStack(Items.CLOCK);
        }
        if (key.contains("ping") || key.contains("insignias")) {
            return new ItemStack(Items.ENDER_PEARL);
        }
        if (key.contains("cps") || key.contains("combo") || key.contains("combate")) {
            return new ItemStack(Items.DIAMOND_SWORD);
        }
        if (key.contains("keystroke")) {
            return new ItemStack(Items.OAK_SIGN);
        }
        if (key.contains("coord")) {
            return new ItemStack(Items.COMPASS);
        }
        if (key.contains("armadura")) {
            return new ItemStack(Items.DIAMOND_CHESTPLATE);
        }
        if (key.contains("contador") || key.contains("bloque")) {
            return new ItemStack(Items.STONE);
        }
        if (key.contains("objeto") || key.contains("mano")) {
            return new ItemStack(Items.STICK);
        }
        if (key.contains("bedwars") || key.contains("recursos")) {
            return new ItemStack(Items.IRON_INGOT);
        }
        if (key.contains("fondo") || key.contains("transparente")) {
            return new ItemStack(Items.GLASS);
        }
        if (key.contains("hardware")) {
            return new ItemStack(Items.REDSTONE);
        }
        if (key.contains("musica")) {
            return new ItemStack(Items.MUSIC_DISC_CAT);
        }
        if (key.contains("pocion")) {
            return new ItemStack(Items.POTION);
        }
        if (key.contains("brújula") || key.contains("brujula")) {
            return new ItemStack(Items.RECOVERY_COMPASS);
        }
        if (key.contains("hitbox") || key.contains("outline")) {
            return new ItemStack(Items.LAPIS_LAZULI);
        }
        if (key.contains("hurt")) {
            return new ItemStack(Items.GOLDEN_APPLE);
        }
        if (key.contains("fire") || key.contains("fuego")) {
            return new ItemStack(Items.FLINT_AND_STEEL);
        }
        if (key.contains("physics") || key.contains("item")) {
            return new ItemStack(Items.SLIME_BALL);
        }
        if (key.contains("tnt")) {
            return new ItemStack(Items.TNT);
        }
        if (key.contains("reach")) {
            return new ItemStack(Items.FISHING_ROD);
        }
        if (key.contains("servidor")) {
            return new ItemStack(Items.PLAYER_HEAD);
        }
        if (key.contains("sprint")) {
            return new ItemStack(Items.LEATHER_BOOTS);
        }
        if (key.contains("sneak")) {
            return new ItemStack(Items.LEATHER_LEGGINGS);
        }
        if (key.contains("pantalla") || key.contains("bordes")) {
            return new ItemStack(Items.ITEM_FRAME);
        }
        if (key.contains("fullbright")) {
            return new ItemStack(Items.GLOWSTONE);
        }
        if (key.contains("fov")) {
            return new ItemStack(Items.SPYGLASS);
        }
        if (key.contains("freelook")) {
            return new ItemStack(Items.ENDER_EYE);
        }
        if (key.contains("anim")) {
            return new ItemStack(Items.BOOK);
        }
        if (key.contains("titulo")) {
            return new ItemStack(Items.WRITABLE_BOOK);
        }
        if (key.contains("chat")) {
            return new ItemStack(Items.WRITTEN_BOOK);
        }
        if (key.contains("scoreboard")) {
            return new ItemStack(Items.OAK_HANGING_SIGN);
        }
        if (key.contains("boost") || key.contains("fps bajo")) {
            return new ItemStack(Items.NETHER_STAR);
        }
        if (key.contains("cull")) {
            return new ItemStack(Items.SPYGLASS);
        }
        if (key.contains("particul")) {
            return new ItemStack(Items.BLAZE_POWDER);
        }
        if (key.contains("memoria")) {
            return new ItemStack(Items.BUCKET);
        }
        if (key.contains("crosshair")) {
            return new ItemStack(Items.TARGET);
        }
        if (key.contains("hypixel") || key.contains("quick")) {
            return new ItemStack(Items.NETHERITE_SWORD);
        }
        if (key.contains("texture") || key.contains("pack")) {
            return new ItemStack(Items.PAINTING);
        }
        if (key.contains("nick")) {
            return new ItemStack(Items.NAME_TAG);
        }
        if (key.contains("cama")) {
            return new ItemStack(Items.RED_BED);
        }
        if (key.contains("equipo") || key.contains("color")) {
            return new ItemStack(Items.CYAN_DYE);
        }
        if (key.contains("tema")) {
            return new ItemStack(Items.GLOW_INK_SAC);
        }
        return ItemStack.EMPTY;
    }

    private static int accentFor(String label) {
        return switch (Math.floorMod(label.hashCode(), 6)) {
            case 0 -> 0x1A2233;
            case 1 -> 0x221A33;
            case 2 -> 0x1A3328;
            case 3 -> 0x332A1A;
            case 4 -> 0x1A2833;
            default -> 0x281A33;
        };
    }

    private static String markFor(String label) {
        String[] parts = label.trim().split("\\s+");
        if (parts.length >= 2) {
            return ("" + parts[0].charAt(0) + parts[1].charAt(0)).toUpperCase(Locale.ROOT);
        }
        return label.length() >= 2 ? label.substring(0, 2).toUpperCase(Locale.ROOT) : label.substring(0, 1).toUpperCase(Locale.ROOT);
    }
}
