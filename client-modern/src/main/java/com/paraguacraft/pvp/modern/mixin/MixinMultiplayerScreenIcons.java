package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.core.ServerIconBootstrap;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.multiplayer.MultiplayerScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(MultiplayerScreen.class)
public class MixinMultiplayerScreenIcons {

    @Inject(method = "init", at = @At("RETURN"))
    private void paraguacraft$fetchMissingIcons(CallbackInfo ci) {
        ServerIconBootstrap.ensureIcons(MinecraftClient.getInstance());
    }
}
