package com.paraguacraft.pvp.gui.theme;

/**
 * Curvas de animación para hover / transiciones (estilo clientes premium).
 */
public final class UiEasing {

    private UiEasing() {}

    public static float easeOutCubic(float t) {
        t = clamp(t);
        return 1f - (float) Math.pow(1 - t, 3);
    }

    public static float easeOutQuad(float t) {
        t = clamp(t);
        return 1f - (1f - t) * (1f - t);
    }

    /** Interpolación suave hacia un objetivo (p. ej. hover 0→1). */
    public static float approach(float current, float target, float speed) {
        if (current < target) {
            return Math.min(target, current + speed);
        }
        return Math.max(target, current - speed);
    }

    private static float clamp(float t) {
        return t < 0f ? 0f : (t > 1f ? 1f : t);
    }
}
