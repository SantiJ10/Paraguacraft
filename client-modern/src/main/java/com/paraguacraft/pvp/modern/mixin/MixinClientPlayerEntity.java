package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.mixin.KeyBindingAccessor;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.FreelookManager;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.ClientPlayerEntity;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import org.lwjgl.glfw.GLFW;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Toggle sprint como 1.8.9:
 * - {@link ModernConfig#toggleSprint}: simula la tecla de sprint (modo virtual).
 * - {@link ModernConfig#toggleSprintLegacy}: W activa sprint con {@code setSprinting}.
 */
@Mixin(ClientPlayerEntity.class)
public class MixinClientPlayerEntity {

    @Inject(method = "tickMovement", at = @At("HEAD"))
    private void paraguacraft$sprintHead(CallbackInfo ci) {
        ClientPlayerEntity player = (ClientPlayerEntity) (Object) this;
        if (FreelookManager.active) {
            player.setYaw(FreelookManager.bodyYaw);
            player.lastYaw = FreelookManager.bodyYaw;
        }
        if (!ModernConfig.toggleSprint) {
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

        if (ModernConfig.toggleSprint) {
            KeyBinding sprint = client.options.sprintKey;
            InputUtil.Key bound = ((KeyBindingAccessor) sprint).paraguacraft$getBoundKey();
            long window = client.getWindow().getHandle();
            boolean physical = bound.getCategory() == InputUtil.Type.KEYSYM
                ? GLFW.glfwGetKey(window, bound.getCode()) == GLFW.GLFW_PRESS
                : sprint.isPressed();
            KeyBinding.setKeyPressed(bound, physical);
        }

        if (!ModernConfig.toggleSprintLegacy) {
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
