package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.ClientPlayerEntity;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Toggle sprint virtual (Lunar) + legacy opcional al final del tick. */
@Mixin(ClientPlayerEntity.class)
public class MixinClientPlayerEntity {

    @Inject(method = "tickMovement", at = @At("HEAD"))
    private void paraguacraft$virtualKeysHead(CallbackInfo ci) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.currentScreen != null) {
            return;
        }
        if (ModernConfig.toggleSprint) {
            client.options.sprintKey.setPressed(true);
        }
    }

    @Inject(method = "tickMovement", at = @At("RETURN"))
    private void paraguacraft$virtualKeysReturn(CallbackInfo ci) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.currentScreen != null) {
            return;
        }

        if (ModernConfig.toggleSprint) {
            KeyBinding sprintKey = client.options.sprintKey;
            InputUtil.Key bound = ((KeyBindingAccessor) sprintKey).paraguacraft$getBoundKey();
            boolean physical = InputUtil.isKeyPressed(client.getWindow(), bound.getCode());
            sprintKey.setPressed(physical);
        }

        if (!ModernConfig.toggleSprintLegacy) {
            return;
        }
        ClientPlayerEntity player = (ClientPlayerEntity) (Object) this;
        if (player.input.hasForwardMovement()
            && !player.isSneaking()
            && !player.isUsingItem()
            && player.getHungerManager().getFoodLevel() > 6) {
            player.setSprinting(true);
        }
    }
}
