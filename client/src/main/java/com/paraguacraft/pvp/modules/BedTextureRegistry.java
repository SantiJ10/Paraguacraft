package com.paraguacraft.pvp.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.texture.TextureAtlasSprite;
import net.minecraft.client.renderer.texture.TextureMap;
import net.minecraft.item.EnumDyeColor;
import net.minecraftforge.client.event.TextureStitchEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;

import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;

/**
 * Sprites de cama recoloreados por equipo (estilo Lunar/Badlion).
 * Se registran en Pre apuntando a la textura vanilla (evita FileNotFound) y se
 * recolorean en Post cuando el atlas ya cargó los píxeles.
 */
public final class BedTextureRegistry {

    private static final String[] PARTS = {
        "bed_feet_end",
        "bed_feet_top",
        "bed_head_end",
        "bed_head_top"
    };

    private static final Map<String, String> VANILLA_TO_KEY = new HashMap<String, String>();
    private static final Set<String> READY_SPRITES = new HashSet<String>();

    private static Field framesField;
    private static Field basePathField;
    private static Field uploadedSpritesField;

    static {
        for (String part : PARTS) {
            VANILLA_TO_KEY.put(part, part);
            VANILLA_TO_KEY.put("minecraft:blocks/" + part, part);
            VANILLA_TO_KEY.put("blocks/" + part, part);
        }
    }

    public BedTextureRegistry() {}

    @SubscribeEvent
    public void onTextureStitchPre(TextureStitchEvent.Pre event) {
        if (event.map != Minecraft.getMinecraft().getTextureMapBlocks()) {
            return;
        }
        READY_SPRITES.clear();
        Map<String, TextureAtlasSprite> uploaded = uploadedSprites(event.map);
        if (uploaded == null) {
            return;
        }
        for (EnumDyeColor dye : EnumDyeColor.values()) {
            for (String part : PARTS) {
                String iconName = iconName(dye, part);
                if (uploaded.containsKey(iconName)) {
                    continue;
                }
                TextureAtlasSprite sprite = newSprite(iconName);
                if (sprite == null) {
                    continue;
                }
                setBasePath(sprite, "blocks/" + part);
                uploaded.put(iconName, sprite);
            }
        }
    }

    @SubscribeEvent
    public void onTextureStitchPost(TextureStitchEvent.Post event) {
        if (event.map != Minecraft.getMinecraft().getTextureMapBlocks()) {
            return;
        }
        READY_SPRITES.clear();
        Map<String, TextureAtlasSprite> uploaded = uploadedSprites(event.map);
        if (uploaded == null) {
            return;
        }
        for (EnumDyeColor dye : EnumDyeColor.values()) {
            float[] rgb = BedColorHelper.dyeToRgb(dye);
            for (String part : PARTS) {
                String iconName = iconName(dye, part);
                TextureAtlasSprite sprite = uploaded.get(iconName);
                if (sprite == null || sprite.getIconWidth() <= 0) {
                    continue;
                }
                if (recolorSpritePixels(sprite, rgb)) {
                    READY_SPRITES.add(iconName);
                }
            }
        }
    }

    public static String mapVanillaSprite(String spriteName) {
        if (!ModConfig.coloredBeds) {
            return null;
        }
        EnumDyeColor dye = BedRenderContext.get();
        if (dye == null) {
            return null;
        }
        String part = VANILLA_TO_KEY.get(spriteName);
        if (part == null && spriteName != null) {
            for (String p : PARTS) {
                if (spriteName.endsWith(p)) {
                    part = p;
                    break;
                }
            }
        }
        if (part == null) {
            return null;
        }
        String iconName = iconName(dye, part);
        if (!READY_SPRITES.contains(iconName)) {
            return null;
        }
        return iconName;
    }

    private static String iconName(EnumDyeColor dye, String part) {
        return "paraguacraft:beds/" + dye.getName() + "_" + part;
    }

    private static TextureAtlasSprite newSprite(String iconName) {
        try {
            Constructor<TextureAtlasSprite> c = TextureAtlasSprite.class.getDeclaredConstructor(String.class);
            c.setAccessible(true);
            return c.newInstance(iconName);
        } catch (Exception ignored) {
            return null;
        }
    }

    private static boolean recolorSpritePixels(TextureAtlasSprite sprite, float[] teamRgb) {
        try {
            Field f = framesField();
            f.setAccessible(true);
            int[][] frames = (int[][]) f.get(sprite);
            if (frames == null || frames.length == 0 || frames[0] == null) {
                return false;
            }
            int tr = (int) (teamRgb[0] * 255);
            int tg = (int) (teamRgb[1] * 255);
            int tb = (int) (teamRgb[2] * 255);
            int[] pixels = frames[0];
            int[] out = new int[pixels.length];
            for (int i = 0; i < pixels.length; i++) {
                int argb = pixels[i];
                int a = (argb >> 24) & 0xFF;
                if (a < 8) {
                    out[i] = argb;
                    continue;
                }
                int r = (argb >> 16) & 0xFF;
                int g = (argb >> 8) & 0xFF;
                int b = argb & 0xFF;
                if (r > 90 && r > g + 25 && r > b + 25) {
                    float lum = (0.299F * r + 0.587F * g + 0.114F * b) / 255.0F;
                    lum = Math.max(0.35F, Math.min(1.0F, lum));
                    int nr = clamp((int) (tr * lum));
                    int ng = clamp((int) (tg * lum));
                    int nb = clamp((int) (tb * lum));
                    out[i] = (a << 24) | (nr << 16) | (ng << 8) | nb;
                } else {
                    out[i] = argb;
                }
            }
            frames[0] = out;
            f.set(sprite, frames);
            return true;
        } catch (Exception ignored) {
            return false;
        }
    }

    @SuppressWarnings("unchecked")
    private static Map<String, TextureAtlasSprite> uploadedSprites(TextureMap map) {
        try {
            Field f = uploadedSpritesField();
            f.setAccessible(true);
            return (Map<String, TextureAtlasSprite>) f.get(map);
        } catch (Exception ignored) {
            return null;
        }
    }

    private static void setBasePath(TextureAtlasSprite sprite, String path) {
        try {
            Field f = basePathField();
            f.setAccessible(true);
            f.set(sprite, path);
        } catch (Exception ignored) {
        }
    }

    private static Field framesField() throws NoSuchFieldException {
        if (framesField != null) {
            return framesField;
        }
        try {
            framesField = TextureAtlasSprite.class.getDeclaredField("framesTextureData");
        } catch (NoSuchFieldException e) {
            framesField = TextureAtlasSprite.class.getDeclaredField("field_110976_a");
        }
        return framesField;
    }

    private static Field basePathField() throws NoSuchFieldException {
        if (basePathField != null) {
            return basePathField;
        }
        try {
            basePathField = TextureAtlasSprite.class.getDeclaredField("basePath");
        } catch (NoSuchFieldException e) {
            basePathField = TextureAtlasSprite.class.getDeclaredField("field_110973_f");
        }
        return basePathField;
    }

    private static Field uploadedSpritesField() throws NoSuchFieldException {
        if (uploadedSpritesField != null) {
            return uploadedSpritesField;
        }
        try {
            uploadedSpritesField = TextureMap.class.getDeclaredField("mapUploadedSprites");
        } catch (NoSuchFieldException e) {
            uploadedSpritesField = TextureMap.class.getDeclaredField("field_110574_e");
        }
        return uploadedSpritesField;
    }

    private static int clamp(int v) {
        return Math.max(0, Math.min(255, v));
    }
}
