package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.modules.BedTextureRegistry;
import net.minecraft.client.renderer.texture.TextureAtlasSprite;
import net.minecraft.client.renderer.texture.TextureMap;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** Redirige sprites de cama vanilla a los recoloreados por equipo (por cama). */
@Mixin(TextureMap.class)
public class MixinTextureMapBed {

    @Inject(method = "getAtlasSprite", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$coloredBedSprite(String spriteName, CallbackInfoReturnable<TextureAtlasSprite> cir) {
        if (spriteName == null || spriteName.startsWith("paraguacraft:")) {
            return;
        }
        String alt = BedTextureRegistry.mapVanillaSprite(spriteName);
        if (alt == null) {
            return;
        }
        TextureAtlasSprite sprite = ((TextureMap) (Object) this).getAtlasSprite(alt);
        if (sprite != null && sprite.getIconName() != null && !sprite.getIconName().equals("missingno")) {
            cir.setReturnValue(sprite);
        }
    }
}
