package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.cosmetics.NametagOverlay;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.ingame.AbstractFurnaceScreen;
import net.minecraft.client.gui.screen.ingame.GenericContainerScreen;
import net.minecraft.client.gui.screen.ingame.HandledScreen;
import net.minecraft.client.gui.screen.ingame.InventoryScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(HandledScreen.class)
public abstract class MixinHandledScreenPlayerPreview {

    @Inject(method = "render", at = @At("TAIL"))
    private void paraguacraft$playerPreview(DrawContext context, int mouseX, int mouseY, float delta, CallbackInfo ci) {
        HandledScreen<?> self = (HandledScreen<?>) (Object) this;
        if (self instanceof InventoryScreen) {
            return;
        }
        if (!(self instanceof GenericContainerScreen) && !(self instanceof AbstractFurnaceScreen)) {
            return;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.player == null) {
            return;
        }
        HandledScreenAccessor acc = (HandledScreenAccessor) this;
        int guiLeft = acc.paraguacraft$getGuiX();
        int guiTop = acc.paraguacraft$getGuiY();
        int xSize = acc.paraguacraft$getBackgroundWidth();
        int feetY = guiTop + 80;
        int feetX = guiLeft >= 70 ? guiLeft - 40 : guiLeft + xSize + 40;
        NametagOverlay.drawPreview(context, feetX, feetY, mouseX, mouseY, client.player);
    }
}
