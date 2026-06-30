package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.entity.EntityPlayerSP;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.objectweb.asm.Opcodes;

/**
 * Aplica toggle sneak/sprint DESPUÉS de {@code updatePlayerMoveState}.
 * Si se hace antes (p. ej. LivingUpdateEvent), vanilla pisa movementInput cada tick.
 */
@Mixin(EntityPlayerSP.class)
public class MixinEntityPlayerSPQoL {

    @Inject(
        method = "onLivingUpdate",
        at = @At(
            value = "FIELD",
            target = "Lnet/minecraft/client/entity/EntityPlayerSP;moveForward:F",
            opcode = Opcodes.PUTFIELD,
            shift = At.Shift.AFTER
        )
    )
    private void paraguacraft$applyMovementToggles(CallbackInfo ci) {
        EntityPlayerSP player = (EntityPlayerSP) (Object) this;
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.currentScreen != null) {
            return;
        }

        if (ModConfig.toggleSneak && ModConfig.isSneakingToggled) {
            player.movementInput.sneak = true;
        }

        if (ModConfig.toggleSprintActive
                && player.movementInput.moveForward > 0.0F
                && !player.isSneaking()
                && !player.isUsingItem()
                && player.getFoodStats().getFoodLevel() > 6
                && !player.isCollidedHorizontally) {
            player.setSprinting(true);
        }
    }
}
