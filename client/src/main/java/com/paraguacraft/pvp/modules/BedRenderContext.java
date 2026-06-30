package com.paraguacraft.pvp.modules;

import net.minecraft.item.EnumDyeColor;

/** Contexto por hilo al compilar/renderizar una cama (color según lana/equipo). */
public final class BedRenderContext {

    private static final ThreadLocal<EnumDyeColor> CURRENT = new ThreadLocal<EnumDyeColor>();

    private BedRenderContext() {}

    public static void set(EnumDyeColor dye) {
        CURRENT.set(dye);
    }

    public static EnumDyeColor get() {
        return CURRENT.get();
    }

    public static void clear() {
        CURRENT.remove();
    }

    public static EnumDyeColor dyeFromRgb(float[] rgb) {
        if (rgb == null) {
            return null;
        }
        EnumDyeColor best = null;
        float bestDist = Float.MAX_VALUE;
        for (EnumDyeColor dye : EnumDyeColor.values()) {
            float[] ref = BedColorHelper.dyeToRgb(dye);
            float dr = ref[0] - rgb[0];
            float dg = ref[1] - rgb[1];
            float db = ref[2] - rgb[2];
            float dist = dr * dr + dg * dg + db * db;
            if (dist < bestDist) {
                bestDist = dist;
                best = dye;
            }
        }
        return best;
    }
}
