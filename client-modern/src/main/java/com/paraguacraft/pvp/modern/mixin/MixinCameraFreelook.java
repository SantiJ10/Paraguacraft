package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.core.FreelookManager;
import net.minecraft.client.render.Camera;
import net.minecraft.entity.Entity;
import net.minecraft.world.World;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Camera.class)
public abstract class MixinCameraFreelook {

    @Inject(method = "update", at = @At("HEAD"))
    private void paraguacraft$freelookApply(
        World area,
        Entity focusedEntity,
        boolean thirdPerson,
        boolean inverseView,
        float tickDelta,
        CallbackInfo ci
    ) {
        FreelookManager.applyCameraOverride(focusedEntity);
    }

    @Inject(method = "update", at = @At("RETURN"))
    private void paraguacraft$freelookRestore(
        World area,
        Entity focusedEntity,
        boolean thirdPerson,
        boolean inverseView,
        float tickDelta,
        CallbackInfo ci
    ) {
        FreelookManager.restoreCameraOverride(focusedEntity);
    }
}
