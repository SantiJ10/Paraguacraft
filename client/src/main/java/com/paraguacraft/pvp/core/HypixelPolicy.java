package com.paraguacraft.pvp.core;

/**
 * Politica de cumplimiento Hypixel — todos los mods del cliente son solo visuales/HUD.
 * No modifican paquetes, alcance de golpe, movimiento ni ventaja competitiva.
 */
public final class HypixelPolicy {

    private HypixelPolicy() {}

    /** Solo lectura/visual — permitido en Hypixel (HUD, OptiFine, chat, perspective). */
    public static final String CATEGORY_COSMETIC = "cosmetic";

    /** Entrenamiento local — muestra datos de TUS acciones, no altera el juego. */
    public static final String CATEGORY_TRAINING = "training";

    public static boolean isAllowedOnHypixel(String category) {
        return CATEGORY_COSMETIC.equals(category) || CATEGORY_TRAINING.equals(category);
    }

    public static String modCategory(int modId) {
        switch (modId) {
            case 47:
            case 48:
                return CATEGORY_TRAINING;
            default:
                return CATEGORY_COSMETIC;
        }
    }

    public static String complianceNote() {
        return "Solo mods cosmeticos y HUD. Sin reach hack, autoclicker, xray ni macros.";
    }
}
