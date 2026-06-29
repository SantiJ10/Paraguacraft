package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.modules.FreelookManager;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.renderer.EntityRenderer;
import net.minecraft.entity.Entity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(EntityRenderer.class)
public class MixinEntityRendererFreelook {

    @Inject(method = "updateCameraAndRender", at = @At("HEAD"))
    private void paraguacraft$updateFreelookMouse(float partialTicks, long nanoTime, CallbackInfo ci) {
        FreelookManager.updateMouse();
    }

    @Redirect(
        method = "orientCamera",
        at = @At(value = "FIELD", target = "Lnet/minecraft/entity/Entity;rotationYaw:F", ordinal = 0)
    )
    private float paraguacraft$freelookYaw(Entity entity) {
        if (ModConfig.freelookEnabled && FreelookManager.active) {
            return FreelookManager.cameraYaw;
        }
        return entity.rotationYaw;
    }

    @Redirect(
        method = "orientCamera",
        at = @At(value = "FIELD", target = "Lnet/minecraft/entity/Entity;rotationPitch:F", ordinal = 0)
    )
    private float paraguacraft$freelookPitch(Entity entity) {
        if (ModConfig.freelookEnabled && FreelookManager.active) {
            return FreelookManager.cameraPitch;
        }
        return entity.rotationPitch;
    }
}
