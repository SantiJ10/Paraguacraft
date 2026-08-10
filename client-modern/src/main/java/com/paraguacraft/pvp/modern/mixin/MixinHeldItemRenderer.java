package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.animations.OldAnimations;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
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

/**
 * Swing + blockhit espada/eje + comer/beber estilo 1.7 en primera persona.
 * El blockhit no usa UseAction.BLOCK (inexistente en espadas 1.9+): usa RMB visual.
 */
@Mixin(HeldItemRenderer.class)
public abstract class MixinHeldItemRenderer {

    @Inject(method = "applySwingOffset", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$oldSwing(MatrixStack matrices, Arm arm, float swingProgress, CallbackInfo ci) {
        if (!OldAnimations.enabled()) {
            return;
        }
        PlayerEntity player = MinecraftClient.getInstance().player;
        if (player != null) {
            Hand hand = arm == player.getMainArm() ? Hand.MAIN_HAND : Hand.OFF_HAND;
            ItemStack stack = player.getStackInHand(hand);
            // Durante blockhit el swing se mezcla en applySwordBlockPose.
            if (OldAnimations.wantsSwordBlockhit(player, hand, stack)) {
                ci.cancel();
                return;
            }
        }
        OldAnimations.applySwingRotation17(matrices, arm, swingProgress);
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
    private void paraguacraft$oldSwordBlockhit(
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
        // Escala viewmodel: armas/tools vs recursos/bloques vs mano vacía (config en Mod Menu).
        float vm = ModernConfig.viewmodelScaleFor(item);
        if (vm != 1.0F) {
            matrices.scale(vm, vm, vm);
        }
        // Bajar un poco el viewmodel (espadas/herramientas se veían muy altas).
        if (item != null && !item.isEmpty() && OldAnimations.isMeleeWeapon(item)) {
            matrices.translate(0.0F, -0.08F, 0.02F);
        }
        if (!OldAnimations.enabled() || player == null || item == null || item.isEmpty()) {
            return;
        }
        if (!OldAnimations.wantsSwordBlockhit(player, hand, item)) {
            return;
        }
        Arm arm = hand == Hand.MAIN_HAND ? player.getMainArm() : player.getMainArm().getOpposite();
        OldAnimations.applySwordBlockPose(matrices, arm, swingProgress);
    }
}
