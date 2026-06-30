package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.entity.EntityPlayerSP;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Toggle sneak/sprint al final de {@code onLivingUpdate}, después de que vanilla
 * procesó input y sneak/sprint, para no ser pisados en el mismo tick.
 */
@Mixin(EntityPlayerSP.class)
public class MixinEntityPlayerSPQoL {

    @Inject(method = "onLivingUpdate", at = @At("RETURN"))
    private void paraguacraft$applyMovementToggles(CallbackInfo ci) {
        EntityPlayerSP player = (EntityPlayerSP) (Object) this;
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer != player || mc.currentScreen != null) {
            return;
        }

        if (ModConfig.toggleSneak && ModConfig.isSneakingToggled) {
            player.movementInput.sneak = true;
            player.setSneaking(true);
        }

        if (!ModConfig.toggleSprintActive) {
            return;
        }

        if (player.movementInput.moveForward <= 0.0F
                || player.isSneaking()
                || player.isUsingItem()
                || player.getFoodStats().getFoodLevel() <= 6) {
            return;
        }
        player.setSprinting(true);
    }
}
