package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.render.WorldRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

/** Hitbox azul cyan estilo Paraguacraft 1.8.9. */
@Mixin(WorldRenderer.class)
public class MixinWorldRendererOutline {

    @ModifyVariable(method = "drawBlockOutline", at = @At("HEAD"), ordinal = 0, argsOnly = true)
    private int paraguacraft$cyanColor(int color) {
        return ModernConfig.showBlockOutline ? 0xFF00E5FF : color;
    }

    @ModifyVariable(method = "drawBlockOutline", at = @At("HEAD"), ordinal = 1, argsOnly = true)
    private float paraguacraft$thickLines(float lineWidth) {
        return ModernConfig.showBlockOutline ? 3.0F : lineWidth;
    }
}
