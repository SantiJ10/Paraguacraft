package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.FreelookManager;
import net.minecraft.client.network.ClientPlayerEntity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Congela cuerpo en freelook; toggle sprint legacy opcional (desactivado por defecto). */
@Mixin(ClientPlayerEntity.class)
public class MixinClientPlayerEntity {

    @Inject(method = "tickMovement", at = @At("HEAD"))
    private void paraguacraft$freelookBody(CallbackInfo ci) {
        ClientPlayerEntity player = (ClientPlayerEntity) (Object) this;
        if (FreelookManager.active) {
            player.setYaw(FreelookManager.bodyYaw);
            player.setHeadYaw(FreelookManager.bodyYaw);
            player.lastYaw = FreelookManager.bodyYaw;
        }
    }

    @Inject(method = "tickMovement", at = @At("RETURN"))
    private void paraguacraft$sprintLegacy(CallbackInfo ci) {
        if (!ModernConfig.toggleSprintLegacy) {
            return;
        }
        ClientPlayerEntity player = (ClientPlayerEntity) (Object) this;
        net.minecraft.client.MinecraftClient client = net.minecraft.client.MinecraftClient.getInstance();
        if (client.currentScreen != null) {
            return;
        }
        if (player.input.hasForwardMovement()
            && !player.isSneaking()
            && !player.isUsingItem()
            && player.getHungerManager().getFoodLevel() > 6) {
            player.setSprinting(true);
        }
    }
}
