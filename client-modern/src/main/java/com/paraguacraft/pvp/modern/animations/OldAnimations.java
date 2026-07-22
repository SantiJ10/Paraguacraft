package com.paraguacraft.pvp.modern.animations;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.PerformanceConfig;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.item.ItemStack;
import net.minecraft.util.math.MathHelper;

/** Animaciones 1.7 en primera persona (swing, comer/beber, blockhit). */
public final class OldAnimations {

    private OldAnimations() {}

    public static boolean enabled() {
        return ModernConfig.oldAnimations || PerformanceConfig.oldAnimations;
    }

    /** Swing de ataque 1.7 — curva sinusoidal mas agresiva que 1.8+. */
    public static void applySwingRotation17(MatrixStack matrices, float swingProgress) {
        rotateY(matrices, 45.0F);
        float f = MathHelper.sin(swingProgress * swingProgress * (float) Math.PI);
        float f1 = MathHelper.sin(MathHelper.sqrt(swingProgress) * (float) Math.PI);
        rotateY(matrices, f * -20.0F);
        rotateZ(matrices, f1 * -20.0F);
        rotateX(matrices, f1 * -80.0F);
    }

    /** Blockhit clasico 1.7 (escudo / bloqueo con item). */
    public static void applyBlockhit(MatrixStack matrices, float swingProgress) {
        translate(matrices, 0.56F, -0.52F, -0.72F);
        rotateY(matrices, 45.0F);

        float f = MathHelper.sin(swingProgress * swingProgress * (float) Math.PI);
        float f1 = MathHelper.sin(MathHelper.sqrt(swingProgress) * (float) Math.PI);
        rotateY(matrices, f * -20.0F);
        rotateZ(matrices, f1 * -20.0F);
        rotateX(matrices, f1 * -80.0F);
        scale(matrices, 0.4F, 0.4F, 0.4F);

        translate(matrices, -0.5F, 0.2F, 0.0F);
        rotateY(matrices, 30.0F);
        rotateX(matrices, -80.0F);
        rotateY(matrices, 60.0F);
    }

    /** Comer/beber — escala y rotacion de 1.7. */
    public static void applyEatOrDrink(MatrixStack matrices, ItemStack stack, int useCount, float partialTicks) {
        float useDuration = (float) stack.getMaxUseTime(null);
        if (useDuration <= 0.0F) {
            return;
        }
        float used = useDuration - (useCount - partialTicks + 1.0F);
        float progress = used / useDuration;
        float f = 1.0F - progress;
        float f1 = MathHelper.abs(MathHelper.cos(used / useDuration * (float) Math.PI * 0.5F) * 0.1F);
        f1 = f1 * f1 * f1 * f1 * f1;

        translate(matrices, 0.56F, -0.44F - f1 * 0.36F, -0.72F);
        translate(matrices, 0.0F, f * -0.15F, 0.0F);
        rotateY(matrices, f1 * 45.0F);
        rotateZ(matrices, f * -20.0F);
        scale(matrices, 0.4F, 0.4F, 0.4F);
        translate(matrices, -0.5F, 0.2F, 0.0F);
        rotateY(matrices, 30.0F);
        rotateX(matrices, -80.0F);
        rotateY(matrices, 60.0F);
    }

    private static void translate(MatrixStack matrices, float x, float y, float z) {
        matrices.translate(x, y, z);
    }

    private static void scale(MatrixStack matrices, float x, float y, float z) {
        matrices.scale(x, y, z);
    }

    private static void rotateX(MatrixStack matrices, float degrees) {
        matrices.multiply(net.minecraft.util.math.RotationAxis.POSITIVE_X.rotationDegrees(degrees));
    }

    private static void rotateY(MatrixStack matrices, float degrees) {
        matrices.multiply(net.minecraft.util.math.RotationAxis.POSITIVE_Y.rotationDegrees(degrees));
    }

    private static void rotateZ(MatrixStack matrices, float degrees) {
        matrices.multiply(net.minecraft.util.math.RotationAxis.POSITIVE_Z.rotationDegrees(degrees));
    }
}
