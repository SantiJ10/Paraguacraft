package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.animations.OldAnimations;
import net.minecraft.client.network.AbstractClientPlayerEntity;
import net.minecraft.client.render.item.HeldItemRenderer;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.entity.player.PlayerEntity;
import net.minecraft.item.ItemStack;
import net.minecraft.util.Arm;
import net.minecraft.util.Hand;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Swing, comer/beber y blockhit estilo 1.7 en primera persona. */
@Mixin(HeldItemRenderer.class)
public abstract class MixinHeldItemRenderer {

    @Inject(method = "applySwingOffset", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$oldSwing(MatrixStack matrices, Arm arm, float swingProgress, CallbackInfo ci) {
        if (!OldAnimations.enabled()) {
            return;
        }
        OldAnimations.applySwingRotation17(matrices, swingProgress);
        ci.cancel();
    }

    @Inject(method = "applyEatOrDrinkTransformation", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$oldEatOrDrink(
        MatrixStack matrices,
        float tickProgress,
        Arm arm,
        ItemStack stack,
        PlayerEntity player,
        CallbackInfo ci
    ) {
        if (!OldAnimations.enabled() || player == null || stack == null || stack.isEmpty()) {
            return;
        }
        if (player.isUsingItem() && player.getActiveItem() == stack && player.getItemUseTimeLeft() > 0) {
            int useCount = player.getItemUseTimeLeft();
            OldAnimations.applyEatOrDrink(matrices, stack, useCount, tickProgress);
            ci.cancel();
        }
    }

    @Inject(
        method = "renderFirstPersonItem",
        at = @At(
            value = "INVOKE",
            target = "Lnet/minecraft/client/render/item/HeldItemRenderer;applyEquipOffset(Lnet/minecraft/client/util/math/MatrixStack;Lnet/minecraft/util/Arm;F)V",
            shift = At.Shift.AFTER
        )
    )
    private void paraguacraft$oldBlockhit(
        AbstractClientPlayerEntity player,
        float tickProgress,
        float pitch,
        Hand hand,
        float swingProgress,
        ItemStack item,
        float equipProgress,
        MatrixStack matrices,
        net.minecraft.client.render.command.OrderedRenderCommandQueue queue,
        int light,
        CallbackInfo ci
    ) {
        if (!OldAnimations.enabled() || player == null || !player.isUsingItem()) {
            return;
        }
        ItemStack active = player.getActiveItem();
        if (active == null || active.isEmpty()) {
            return;
        }
        if (active.getUseAction() != net.minecraft.item.consume.UseAction.BLOCK) {
            return;
        }
        if (player.getActiveHand() != hand) {
            return;
        }
        OldAnimations.applyBlockhit(matrices, swingProgress);
    }
}
