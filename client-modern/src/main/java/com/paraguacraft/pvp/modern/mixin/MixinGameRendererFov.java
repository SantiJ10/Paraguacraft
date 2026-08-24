package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.render.GameRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** FOV estático: anula el multiplicador de speed/sprint (sin tocar Zoomify). */
@Mixin(GameRenderer.class)
public abstract class MixinGameRendererFov {

    @Shadow
    private float fovMultiplier;

    @Shadow
    private float lastFovMultiplier;

    @Inject(method = "updateFovMultiplier", at = @At("TAIL"), require = 0)
    private void paraguacraft$disableSpeedFov(CallbackInfo ci) {
        if (ModernConfig.dynamicFov) {
            return;
        }
        this.fovMultiplier = 1.0F;
        this.lastFovMultiplier = 1.0F;
    }
}
