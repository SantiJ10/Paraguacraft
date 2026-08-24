package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.cosmetics.NametagOverlay;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.ingame.InventoryScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(InventoryScreen.class)
public abstract class MixinInventoryScreenOverlay {

    @Inject(method = "render", at = @At("TAIL"))
    private void paraguacraft$playerTag(DrawContext context, int mouseX, int mouseY, float delta, CallbackInfo ci) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.player == null) {
            return;
        }
        HandledScreenAccessor acc = (HandledScreenAccessor) this;
        NametagOverlay.drawInventoryTag(context, acc.paraguacraft$getGuiX(), acc.paraguacraft$getGuiY(), client.player);
    }
}
