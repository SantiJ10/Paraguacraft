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
            double sens = client.options.getMouseSensitivity().getValue() * 0.6D + 0.2D;
            sens = sens * sens * sens * 8.0D;
            FreelookManager.addMouseDelta((float) (cursorDeltaX * sens), (float) (cursorDeltaY * sens));
            ci.cancel();
        }
    }
}
