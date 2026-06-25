package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.gui.PanoramaBackground;
import net.minecraft.client.multiplayer.GuiConnecting;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.network.NetworkManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(GuiConnecting.class)
public abstract class MixinGuiConnecting {

    @Shadow private NetworkManager networkManager;

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
    private void paraguacraft$drawConnectingUi(int mouseX, int mouseY, float partialTicks, CallbackInfo ci) {
        GuiScreen self = (GuiScreen) (Object) this;
        String sub = "";
        if (this.networkManager != null && this.networkManager.getRemoteAddress() != null) {
            sub = this.networkManager.getRemoteAddress().toString();
        }
        PanoramaBackground.drawLoading(self, "Conectando al servidor…", sub);
    }

    @Redirect(
        method = "drawScreen",
        at = @At(
            value = "INVOKE",
            target = "Lnet/minecraft/client/gui/GuiScreen;drawCenteredString(Lnet/minecraft/client/gui/FontRenderer;Ljava/lang/String;III)V"
        )
    )
    private void paraguacraft$skipVanillaConnectingText(
        GuiScreen instance,
        net.minecraft.client.gui.FontRenderer fr,
        String text,
        int x,
        int y,
        int color
    ) {
    }
}
