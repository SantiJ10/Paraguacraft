package com.paraguacraft.pvp.modern.animations;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.PerformanceConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.entity.player.PlayerEntity;
import net.minecraft.item.ItemStack;
import net.minecraft.item.consume.UseAction;
import net.minecraft.registry.tag.ItemTags;
import net.minecraft.util.Arm;
import net.minecraft.util.Hand;
import net.minecraft.util.math.MathHelper;
import net.minecraft.util.math.RotationAxis;

/**
 * Animaciones 1.7 en primera persona (1.21+).
 *
 * Nota: en 1.9+ las espadas ya no tienen {@link UseAction#BLOCK}; el blockhit
 * clásico (RMB + swing) es solo visual vía tecla de uso + arma melee.
 */
public final class OldAnimations {

    private OldAnimations() {}

    public static boolean enabled() {
        return ModernConfig.oldAnimations || PerformanceConfig.oldAnimations;
    }

    /** Espada / hacha (melee clásico). */
    public static boolean isMeleeWeapon(ItemStack stack) {
        if (stack == null || stack.isEmpty()) {
            return false;
        }
        return stack.isIn(ItemTags.SWORDS) || stack.isIn(ItemTags.AXES);
    }

    /**
     * Pose “block + hit” 1.7: click derecho + arma en main hand.
     * No reintroduce bloqueo 1.8 (solo matrices de render).
     */
    public static boolean wantsSwordBlockhit(PlayerEntity player, Hand hand, ItemStack stack) {
        if (!enabled() || player == null || stack == null || stack.isEmpty()) {
            return false;
        }
        if (hand != Hand.MAIN_HAND || !isMeleeWeapon(stack)) {
            return false;
        }
        // Escudo real en offhand: dejar el anim vanilla del shield.
        if (player.isUsingItem()
            && player.getActiveHand() == Hand.OFF_HAND
            && !player.getOffHandStack().isEmpty()
            && player.getOffHandStack().getUseAction() == UseAction.BLOCK) {
            return false;
        }
        MinecraftClient mc = MinecraftClient.getInstance();
        return mc != null && mc.options.useKey.isPressed();
    }

    /** Swing de ataque 1.7 — curva sinusoidal más agresiva que 1.8+. */
    public static void applySwingRotation17(MatrixStack matrices, Arm arm, float swingProgress) {
        float side = arm == Arm.RIGHT ? 1.0F : -1.0F;
        float f = MathHelper.sin(swingProgress * swingProgress * (float) Math.PI);
        float f1 = MathHelper.sin(MathHelper.sqrt(swingProgress) * (float) Math.PI);
        // Base 45° + arcos del swing 1.7 (más “taladro” que vanilla modern).
        matrices.multiply(RotationAxis.POSITIVE_Y.rotationDegrees(side * 45.0F));
        matrices.multiply(RotationAxis.POSITIVE_Y.rotationDegrees(side * f * -20.0F));
        matrices.multiply(RotationAxis.POSITIVE_Z.rotationDegrees(side * f1 * -20.0F));
        matrices.multiply(RotationAxis.POSITIVE_X.rotationDegrees(f1 * -80.0F));
        // Un poco más cerca / compacto, sin empujar la mano hacia arriba.
        matrices.translate(side * -0.02F, -0.02F, 0.04F);
        matrices.scale(0.92F, 0.92F, 0.92F);
    }

    /**
     * Blockhit clásico después de {@code applyEquipOffset}.
     * Sin el translate absoluto 0.56/-0.52 (ya lo hizo equip).
     */
    public static void applySwordBlockPose(MatrixStack matrices, Arm arm, float swingProgress) {
        float side = arm == Arm.RIGHT ? 1.0F : -1.0F;
        // Pose block: menos elevación (antes 0.14 subía mucho el arma).
        matrices.translate(side * -0.08F, 0.02F, 0.06F);
        matrices.multiply(RotationAxis.POSITIVE_Y.rotationDegrees(side * 50.0F));
        matrices.multiply(RotationAxis.POSITIVE_X.rotationDegrees(-70.0F));
        matrices.multiply(RotationAxis.POSITIVE_Z.rotationDegrees(side * -25.0F));

        float f = MathHelper.sin(swingProgress * swingProgress * (float) Math.PI);
        float f1 = MathHelper.sin(MathHelper.sqrt(swingProgress) * (float) Math.PI);
        matrices.multiply(RotationAxis.POSITIVE_Y.rotationDegrees(side * f * -20.0F));
        matrices.multiply(RotationAxis.POSITIVE_Z.rotationDegrees(side * f1 * -20.0F));
        matrices.multiply(RotationAxis.POSITIVE_X.rotationDegrees(f1 * -40.0F));
        matrices.scale(0.88F, 0.88F, 0.88F);
    }

    /** Comer/beber — escala y rotación de 1.7. */
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

        matrices.translate(0.0F, -0.06F - f1 * 0.28F, 0.0F);
        matrices.translate(0.0F, f * -0.12F, 0.0F);
        matrices.multiply(RotationAxis.POSITIVE_Y.rotationDegrees(f1 * 45.0F));
        matrices.multiply(RotationAxis.POSITIVE_Z.rotationDegrees(f * -20.0F));
        matrices.scale(0.95F, 0.95F, 0.95F);
    }
}
