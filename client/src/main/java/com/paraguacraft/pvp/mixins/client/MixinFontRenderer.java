package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.gui.theme.TextUtil;
import net.minecraft.client.gui.FontRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

/** Evita que acentos (p. ej. Sí) se dibujen como iconos del atlas vanilla. */
@Mixin(FontRenderer.class)
public abstract class MixinFontRenderer {

    @ModifyVariable(
        method = "drawString(Ljava/lang/String;FFIZ)I",
        at = @At("HEAD"),
        argsOnly = true,
        ordinal = 0
    )
    private String paraguacraft$sanitizeDraw(String text) {
        return TextUtil.sanitizeForMcFont(text);
    }

    @ModifyVariable(
        method = "getStringWidth",
        at = @At("HEAD"),
        argsOnly = true,
        ordinal = 0
    )
    private String paraguacraft$sanitizeWidth(String text) {
        return TextUtil.sanitizeForMcFont(text);
    }
}
