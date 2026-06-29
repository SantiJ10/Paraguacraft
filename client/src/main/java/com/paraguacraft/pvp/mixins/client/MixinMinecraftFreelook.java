package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.modules.FreelookManager;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.entity.EntityPlayerSP;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

/**
 * Desvía la rotación del mouse hacia la cámara libre.
 * {@code Minecraft.runTick} llama a {@code thePlayer.setAngles(deltaX, deltaY)};
 * cuando freelook está activo absorbemos esos deltas en la cámara en vez de
 * aplicarlos al cuerpo del jugador (misma sensibilidad que vanilla).
 */
@Mixin(Minecraft.class)
public class MixinMinecraftFreelook {

    @Redirect(
        method = "runTick",
        at = @At(
            value = "INVOKE",
            target = "Lnet/minecraft/client/entity/EntityPlayerSP;setAngles(FF)V"
        ),
        require = 0
    )
    private void paraguacraft$freelookAbsorb(EntityPlayerSP player, float yaw, float pitch) {
        if (ModConfig.freelookEnabled && FreelookManager.active) {
            FreelookManager.addMouseDelta(yaw, pitch);
        } else {
            player.setAngles(yaw, pitch);
        }
    }
}
