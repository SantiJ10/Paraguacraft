package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.core.ServerIconFetcher;
import net.minecraft.client.gui.GuiMultiplayer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(GuiMultiplayer.class)
public class MixinGuiMultiplayerIcons {

    @Inject(method = "initGui", at = @At("RETURN"))
    private void paraguacraft$fetchMissingIcons(CallbackInfo ci) {
        ServerIconFetcher.ensureIcons();
    }
}
