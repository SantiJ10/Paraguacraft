package com.paraguacraft.pvp.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.texture.TextureAtlasSprite;
import net.minecraft.client.renderer.texture.TextureMap;
import net.minecraft.item.EnumDyeColor;
import net.minecraft.util.ResourceLocation;
import net.minecraftforge.client.event.TextureStitchEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;

import javax.imageio.ImageIO;
import java.awt.image.BufferedImage;
import java.io.InputStream;
import java.lang.reflect.Field;
import java.util.HashMap;
import java.util.Map;

/**
 * Registra sprites de cama recoloreados por equipo (estilo Lunar/Badlion).
 * Cada cama usa el sprite según lana cercana o equipo al compilar el chunk.
 */
public final class BedTextureRegistry {

    private static final String[] PARTS = {
        "bed_feet_end",
        "bed_feet_top",
        "bed_head_end",
        "bed_head_top"
    };

    private static final Map<String, String> VANILLA_TO_KEY = new HashMap<String, String>();
    private static Field framesField;

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
        // 0 = atlas de bloques en 1.8.9
        if (event.map != Minecraft.getMinecraft().getTextureMapBlocks()) {
            return;
        }
        TextureMap map = event.map;
        for (EnumDyeColor dye : EnumDyeColor.values()) {
            float[] rgb = BedColorHelper.dyeToRgb(dye);
            for (String part : PARTS) {
                String key = spriteKey(dye, part);
                ResourceLocation loc = new ResourceLocation("paraguacraft", key);
                TextureAtlasSprite sprite = map.registerSprite(loc);
                BufferedImage img = recolorVanilla(part, rgb);
                if (img != null) {
                    patchSprite(sprite, img);
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
        return "paraguacraft:" + spriteKey(dye, part);
    }

    private static String spriteKey(EnumDyeColor dye, String part) {
        return "beds/" + dye.getName() + "_" + part;
    }

    private static BufferedImage recolorVanilla(String part, float[] teamRgb) {
        try {
            ResourceLocation src = new ResourceLocation("textures/blocks/" + part + ".png");
            InputStream in = Minecraft.getMinecraft().getResourceManager().getResource(src).getInputStream();
            BufferedImage img = ImageIO.read(in);
            in.close();
            if (img == null) {
                return null;
            }
            int tr = (int) (teamRgb[0] * 255);
            int tg = (int) (teamRgb[1] * 255);
            int tb = (int) (teamRgb[2] * 255);
            int w = img.getWidth();
            int h = img.getHeight();
            BufferedImage out = new BufferedImage(w, h, BufferedImage.TYPE_INT_ARGB);
            for (int y = 0; y < h; y++) {
                for (int x = 0; x < w; x++) {
                    int argb = img.getRGB(x, y);
                    int a = (argb >> 24) & 0xFF;
                    if (a < 8) {
                        out.setRGB(x, y, argb);
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
                        out.setRGB(x, y, (a << 24) | (nr << 16) | (ng << 8) | nb);
                    } else {
                        out.setRGB(x, y, argb);
                    }
                }
            }
            return out;
        } catch (Exception ignored) {
            return null;
        }
    }

    private static void patchSprite(TextureAtlasSprite sprite, BufferedImage img) {
        try {
            int w = sprite.getIconWidth() > 0 ? sprite.getIconWidth() : img.getWidth();
            int h = sprite.getIconHeight() > 0 ? sprite.getIconHeight() : img.getHeight();
            if (img.getWidth() != w || img.getHeight() != h) {
                BufferedImage scaled = new BufferedImage(w, h, BufferedImage.TYPE_INT_ARGB);
                java.awt.Graphics2D g = scaled.createGraphics();
                g.drawImage(img, 0, 0, w, h, null);
                g.dispose();
                img = scaled;
            }
            int[] pixels = new int[w * h];
            img.getRGB(0, 0, w, h, pixels, 0, w);
            Field f = framesField();
            f.setAccessible(true);
            int[][] frames = (int[][]) f.get(sprite);
            if (frames == null || frames.length == 0) {
                frames = new int[][] {pixels};
            } else {
                frames[0] = pixels;
            }
            f.set(sprite, frames);
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

    private static int clamp(int v) {
        return Math.max(0, Math.min(255, v));
    }
}
