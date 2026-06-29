package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.modules.FreelookManager;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.EntityRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Aplica los ángulos de la cámara libre durante el posicionamiento de cámara. */
@Mixin(EntityRenderer.class)
public class MixinEntityRendererFreelook {

    @Inject(method = "orientCamera", at = @At("HEAD"))
    private void paraguacraft$freelookApply(float partialTicks, CallbackInfo ci) {
        FreelookManager.applyCameraOverride(Minecraft.getMinecraft());
    }

    @Inject(method = "orientCamera", at = @At("RETURN"))
    private void paraguacraft$freelookRestore(float partialTicks, CallbackInfo ci) {
        FreelookManager.restoreCameraOverride(Minecraft.getMinecraft());
    }
}
