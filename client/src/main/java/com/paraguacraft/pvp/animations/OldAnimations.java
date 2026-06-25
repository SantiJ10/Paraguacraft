package com.paraguacraft.pvp.animations;

import net.minecraft.client.Minecraft;
import net.minecraft.client.entity.EntityPlayerSP;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.ItemRenderer;
import net.minecraft.client.renderer.block.model.ItemCameraTransforms;
import net.minecraft.item.EnumAction;
import net.minecraft.item.ItemStack;
import net.minecraft.util.MathHelper;

/**
 * Matemáticas de animaciones 1.7.10 portadas a 1.8.9.
 * Usadas desde {@link com.paraguacraft.pvp.mixins.MixinItemRenderer} (sin reflexión).
 */
public final class OldAnimations {

    private OldAnimations() {}

    /**
     * Render completo en primera persona estilo 1.7 (solo blockhit / comer).
     * El swing se maneja vía {@code @Redirect} en {@code MixinItemRenderer}.
     */
    public static boolean renderFirstPerson(ItemRenderer renderer, Minecraft mc, float partialTicks) {
        EntityPlayerSP player = mc.thePlayer;
        if (player == null) {
            return false;
        }
        ItemStack stack = player.inventory.getCurrentItem();
        if (stack == null) {
            return false;
        }
        EnumAction action = stack.getItemUseAction();
        int useCount = player.getItemInUseCount();
        if (useCount <= 0) {
            return false;
        }
        GlStateManager.pushMatrix();
        if (action == EnumAction.BLOCK) {
            applyBlockhit(player.getSwingProgress(partialTicks));
            renderer.renderItem(player, stack, ItemCameraTransforms.TransformType.FIRST_PERSON);
            GlStateManager.popMatrix();
            return true;
        }
        if (action == EnumAction.EAT || action == EnumAction.DRINK) {
            applyEatOrDrink(stack, useCount, partialTicks);
            renderer.renderItem(player, stack, ItemCameraTransforms.TransformType.FIRST_PERSON);
            GlStateManager.popMatrix();
            return true;
        }
        GlStateManager.popMatrix();
        return false;
    }

    /** Blockhit clásico 1.7 — espada cruzada al mantener click derecho con espada. */
    public static void applyBlockhit(float swingProgress) {
        GlStateManager.translate(0.56F, -0.52F, -0.72F);
        GlStateManager.rotate(45.0F, 0.0F, 1.0F, 0.0F);

        float f = MathHelper.sin(swingProgress * swingProgress * (float) Math.PI);
        float f1 = MathHelper.sin(MathHelper.sqrt_float(swingProgress) * (float) Math.PI);
        GlStateManager.rotate(f * -20.0F, 0.0F, 1.0F, 0.0F);
        GlStateManager.rotate(f1 * -20.0F, 0.0F, 0.0F, 1.0F);
        GlStateManager.rotate(f1 * -80.0F, 1.0F, 0.0F, 0.0F);
        GlStateManager.scale(0.4F, 0.4F, 0.4F);

        GlStateManager.translate(-0.5F, 0.2F, 0.0F);
        GlStateManager.rotate(30.0F, 0.0F, 1.0F, 0.0F);
        GlStateManager.rotate(-80.0F, 1.0F, 0.0F, 0.0F);
        GlStateManager.rotate(60.0F, 0.0F, 1.0F, 0.0F);
    }

    /** Comer/beber — escala y rotación de 1.7 (más compacto que 1.8). */
    public static void applyEatOrDrink(ItemStack stack, int useCount, float partialTicks) {
        float useDuration = (float) stack.getMaxItemUseDuration();
        float used = useDuration - (useCount - partialTicks + 1.0F);
        float progress = used / useDuration;
        float f = 1.0F - progress;
        float f1 = MathHelper.abs(MathHelper.cos(used / useDuration * (float) Math.PI * 0.5F) * 0.1F);
        f1 = f1 * f1 * f1 * f1 * f1;

        GlStateManager.translate(0.56F, -0.44F - f1 * 0.36F, -0.72F);
        GlStateManager.translate(0.0F, f * -0.15F, 0.0F);
        GlStateManager.rotate(f1 * 45.0F, 0.0F, 1.0F, 0.0F);
        GlStateManager.rotate(f * -20.0F, 0.0F, 0.0F, 1.0F);
        GlStateManager.scale(0.4F, 0.4F, 0.4F);
        GlStateManager.translate(-0.5F, 0.2F, 0.0F);
        GlStateManager.rotate(30.0F, 0.0F, 1.0F, 0.0F);
        GlStateManager.rotate(-80.0F, 1.0F, 0.0F, 0.0F);
        GlStateManager.rotate(60.0F, 0.0F, 1.0F, 0.0F);
    }

    /** Swing de ataque 1.7 — curva sinusoidal más agresiva que 1.8. */
    public static void applySwingRotation17(float swingProgress) {
        GlStateManager.rotate(45.0F, 0.0F, 1.0F, 0.0F);
        float f = MathHelper.sin(swingProgress * swingProgress * (float) Math.PI);
        float f1 = MathHelper.sin(MathHelper.sqrt_float(swingProgress) * (float) Math.PI);
        GlStateManager.rotate(f * -20.0F, 0.0F, 1.0F, 0.0F);
        GlStateManager.rotate(f1 * -20.0F, 0.0F, 0.0F, 1.0F);
        GlStateManager.rotate(f1 * -80.0F, 1.0F, 0.0F, 0.0F);
    }

    /** @deprecated Usar {@link #applySwingRotation17(float)} desde el mixin. */
    public static void applySwing17(float swingProgress) {
        GlStateManager.translate(0.56F, -0.52F, -0.72F);
        applySwingRotation17(swingProgress);
        GlStateManager.scale(0.4F, 0.4F, 0.4F);
    }

    private static void transformSwing17(float swingProgress) {
        applySwingRotation17(swingProgress);
    }
}
