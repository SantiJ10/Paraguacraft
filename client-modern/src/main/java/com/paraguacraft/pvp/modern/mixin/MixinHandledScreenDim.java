package com.paraguacraft.pvp.modern.mixin;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.ingame.CreativeInventoryScreen;
import net.minecraft.client.gui.screen.ingame.HandledScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Ligero velo detras de containers. El look principal viene del resource pack
 * (paneles negros + acentos celestes); no oscurecer de mas el creativo.
 */
@Mixin(HandledScreen.class)
public class MixinHandledScreenDim {

    @Inject(method = "renderBackground", at = @At("TAIL"))
    private void paraguacraft$dimInventory(DrawContext context, int mouseX, int mouseY, float delta, CallbackInfo ci) {
        HandledScreen<?> self = (HandledScreen<?>) (Object) this;
        // Creativo ya trae paneles propios en 1.21; un velo fuerte lo deja ilegible.
        if (self instanceof CreativeInventoryScreen) {
            context.fill(0, 0, self.width, self.height, 0x55000000);
            return;
        }
        // ~40% negro detras del inventario survival/chests
        context.fill(0, 0, self.width, self.height, 0x66000000);
    }
}
