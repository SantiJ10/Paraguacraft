package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.mixin.KeyBindingAccessor;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.ClientPlayerEntity;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Toggle sprint estilo 1.8.9: W activa sprint automáticamente. */
@Mixin(ClientPlayerEntity.class)
public class MixinClientPlayerEntity {

    @Inject(method = "tickMovement", at = @At("HEAD"))
    private void paraguacraft$sprintHead(CallbackInfo ci) {
        if (!ModernConfig.toggleSprint || !ModernConfig.toggleSprintLegacy) {
            return;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.currentScreen != null) {
            return;
        }
        KeyBinding sprint = client.options.sprintKey;
        InputUtil.Key bound = ((KeyBindingAccessor) sprint).paraguacraft$getBoundKey();
        KeyBinding.setKeyPressed(bound, true);
    }

    @Inject(method = "tickMovement", at = @At("RETURN"))
    private void paraguacraft$sprintReturn(CallbackInfo ci) {
        ClientPlayerEntity player = (ClientPlayerEntity) (Object) this;
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.currentScreen != null) {
            return;
        }
        if (ModernConfig.toggleSprint && ModernConfig.toggleSprintLegacy) {
            if (player.input.hasForwardMovement()
                && !player.isSneaking()
                && !player.isUsingItem()
                && player.getHungerManager().getFoodLevel() > 6) {
                player.setSprinting(true);
            }
            KeyBinding sprint = client.options.sprintKey;
            InputUtil.Key bound = ((KeyBindingAccessor) sprint).paraguacraft$getBoundKey();
            KeyBinding.setKeyPressed(bound, sprint.isPressed());
        }
    }
}
