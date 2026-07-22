package com.paraguacraft.pvp.modern.mixin.performance;

import com.paraguacraft.pvp.modern.core.CullHelper;
import net.minecraft.client.render.Frustum;
import net.minecraft.client.render.entity.EntityRenderManager;
import net.minecraft.entity.Entity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** Cull de entidades, armor stands e item frames (paridad 1.8.9). */
@Mixin(EntityRenderManager.class)
public abstract class MixinEntityRenderManagerCull {

    @Inject(method = "shouldRender", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$cullEntities(
        Entity entity,
        Frustum frustum,
        double camX,
        double camY,
        double camZ,
        CallbackInfoReturnable<Boolean> cir
    ) {
        if (CullHelper.shouldCullEntity(entity, frustum, camX, camY, camZ)) {
            cir.setReturnValue(false);
        }
    }
}
