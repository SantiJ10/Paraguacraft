package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.core.FreelookManager;
import net.minecraft.client.render.GameRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import net.minecraft.client.MinecraftClient;
import org.spongepowered.asm.mixin.Shadow;

@Mixin(GameRenderer.class)
public class MixinGameRendererFreelook {

    @Shadow private MinecraftClient client;

    @Inject(method = "updateCameraState", at = @At("HEAD"))
    private void paraguacraft$freelookApply(float tickProgress, CallbackInfo ci) {
        FreelookManager.applyCameraOverride(client);
    }

    @Inject(method = "updateCameraState", at = @At("RETURN"))
    private void paraguacraft$freelookRestore(float tickProgress, CallbackInfo ci) {
        FreelookManager.restoreCameraOverride(client);
    }
}
