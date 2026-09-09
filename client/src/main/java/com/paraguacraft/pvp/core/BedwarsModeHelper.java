package com.paraguacraft.pvp.core;

/**
 * Detecta BedWars. La deteccion vive en {@link GameModeDetector};
 * este helper se mantiene para llamadas existentes.
 */
public final class BedwarsModeHelper {

    private BedwarsModeHelper() {}

    public static boolean isBedwars() {
        return GameModeDetector.current() == GameModeDetector.Mode.BEDWARS;
    }
}
