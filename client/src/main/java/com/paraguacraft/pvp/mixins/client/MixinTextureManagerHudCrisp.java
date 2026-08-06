package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.hud.HudModuleScale;
import net.minecraft.client.renderer.texture.TextureManager;
import net.minecraft.util.ResourceLocation;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Con el HUD escalado, fuerza filtrado nearest en cada bind de textura
 * (texto, items, carátula) para que no se vean borrosos al achicar/agrandar.
 */
@Mixin(TextureManager.class)
public abstract class MixinTextureManagerHudCrisp {

    @Inject(method = "bindTexture", at = @At("RETURN"))
    private void paraguacraft$hudCrispFilter(ResourceLocation resource, CallbackInfo ci) {
        if (HudModuleScale.isCrisp()) {
            HudModuleScale.applyNearestOnBound();
        }
    }
}
