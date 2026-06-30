package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.entity.EntityPlayerSP;
import net.minecraft.client.settings.KeyBinding;
import org.lwjgl.input.Keyboard;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Toggle sneak/sprint: modo virtual (teclas simuladas, patrón Lunar) y modo legacy
 * ({@code setSprinting} al final del tick) coexisten para testeo A/B.
 */
@Mixin(EntityPlayerSP.class)
public class MixinEntityPlayerSPQoL {

    @Inject(method = "onLivingUpdate", at = @At("HEAD"))
    private void paraguacraft$applyVirtualKeysHead(CallbackInfo ci) {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.currentScreen != null) {
            return;
        }

        if (ModConfig.toggleSneak) {
            int sneakCode = mc.gameSettings.keyBindSneak.getKeyCode();
            KeyBinding.setKeyBindState(sneakCode, ModConfig.isSneakingToggled);
        }

        if (ModConfig.toggleSprintActive) {
            KeyBinding.setKeyBindState(mc.gameSettings.keyBindSprint.getKeyCode(), true);
        }
    }

    @Inject(method = "onLivingUpdate", at = @At("RETURN"))
    private void paraguacraft$movementTogglesReturn(CallbackInfo ci) {
        EntityPlayerSP player = (EntityPlayerSP) (Object) this;
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer != player || mc.currentScreen != null) {
            return;
        }

        if (ModConfig.toggleSneak) {
            int sneakCode = mc.gameSettings.keyBindSneak.getKeyCode();
            KeyBinding.setKeyBindState(sneakCode, Keyboard.isKeyDown(sneakCode));
        }

        if (ModConfig.toggleSprintActive) {
            int sprintCode = mc.gameSettings.keyBindSprint.getKeyCode();
            KeyBinding.setKeyBindState(sprintCode, Keyboard.isKeyDown(sprintCode));
        }

        if (!ModConfig.toggleSprintLegacyActive) {
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
