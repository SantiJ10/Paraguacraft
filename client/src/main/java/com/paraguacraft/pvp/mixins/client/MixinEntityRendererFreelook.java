package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.modules.FreelookManager;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.EntityRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(EntityRenderer.class)
public class MixinEntityRendererFreelook {

    @Shadow private Minecraft mc;

    @Inject(method = "updateCameraAndRender", at = @At("HEAD"))
    private void paraguacraft$updateFreelookMouse(float partialTicks, long nanoTime, CallbackInfo ci) {
        FreelookManager.updateMouse();
    }

    @Inject(method = "orientCamera", at = @At("HEAD"))
    private void paraguacraft$freelookApply(float partialTicks, CallbackInfo ci) {
        FreelookManager.applyCameraOverride(mc);
    }

    @Inject(method = "orientCamera", at = @At("RETURN"))
    private void paraguacraft$freelookRestore(CallbackInfo ci) {
        FreelookManager.restoreCameraOverride(mc);
    }
}
