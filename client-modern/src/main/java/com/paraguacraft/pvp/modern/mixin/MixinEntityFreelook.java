package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.FreelookManager;
import net.minecraft.client.MinecraftClient;
import net.minecraft.entity.Entity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Entity.class)
public class MixinEntityFreelook {

    @Inject(method = "changeLookDirection", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$freelookAbsorb(double cursorDeltaX, double cursorDeltaY, CallbackInfo ci) {
        if (!ModernConfig.freelookEnabled || !FreelookManager.active) {
            return;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.player != null && (Object) this == client.player) {
            FreelookManager.addMouseDelta((float) cursorDeltaX, (float) cursorDeltaY);
            ci.cancel();
        }
    }
}
