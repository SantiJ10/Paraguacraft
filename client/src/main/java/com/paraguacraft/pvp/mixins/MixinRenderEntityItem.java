package com.paraguacraft.pvp.mixins;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.resources.model.IBakedModel;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;
import net.minecraft.client.renderer.entity.RenderEntityItem;

@Mixin(RenderEntityItem.class)
public abstract class MixinRenderEntityItem {

    @Redirect(
        method = "doRender(Lnet/minecraft/entity/item/EntityItem;DDDFF)V",
        at = @At(value = "INVOKE", target = "Lnet/minecraft/client/resources/model/IBakedModel;isGui3d()Z")
    )
    private boolean paraguacraft$forceItemRenderMode(IBakedModel model) {
        if (ModConfig.itemPhysics) {
            return false;
        }
        return ModConfig.forceItem3d;
    }
}
