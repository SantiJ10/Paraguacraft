package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.gui.PanoramaBackground;
import net.minecraft.client.gui.GuiDownloadTerrain;
import net.minecraft.client.gui.GuiScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(GuiDownloadTerrain.class)
public abstract class MixinGuiDownloadTerrain {

    @Redirect(
        method = "drawScreen",
        at = @At(
            value = "INVOKE",
            target = "Lnet/minecraft/client/gui/GuiScreen;drawDefaultBackground()V"
        )
    )
    private void paraguacraft$skipDirtBackground(GuiScreen instance) {
    }

    @Inject(method = "drawScreen", at = @At("HEAD"))
    private void paraguacraft$drawLoadingUi(int mouseX, int mouseY, float partialTicks, CallbackInfo ci) {
        PanoramaBackground.drawLoading(
            (GuiScreen) (Object) this,
            "Cargando mundo…",
            "Preparando chunks y recursos"
        );
    }

    @Redirect(
        method = "drawScreen",
        at = @At(
            value = "INVOKE",
            target = "Lnet/minecraft/client/gui/GuiScreen;drawCenteredString(Lnet/minecraft/client/gui/FontRenderer;Ljava/lang/String;III)V"
        )
    )
    private void paraguacraft$skipVanillaLoadingText(
        GuiScreen instance,
        net.minecraft.client.gui.FontRenderer fr,
        String text,
        int x,
        int y,
        int color
    ) {
    }
}
