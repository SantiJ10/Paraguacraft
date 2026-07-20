package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.NickFinderManager;
import net.minecraft.client.gui.hud.PlayerListHud;
import net.minecraft.client.network.PlayerListEntry;
import net.minecraft.text.Text;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(PlayerListHud.class)
public class MixinPlayerListHudNickTeam {

    @Inject(method = "getPlayerName", at = @At("RETURN"), cancellable = true)
    private void paraguacraft$highlightNick(PlayerListEntry entry, CallbackInfoReturnable<Text> cir) {
        if (!ModernConfig.nickFinderEnabled || !NickFinderManager.isActive()) {
            return;
        }
        Text original = cir.getReturnValue();
        if (original == null) {
            return;
        }
        Text highlighted = NickFinderManager.highlightLabel(original);
        if (highlighted != original) {
            cir.setReturnValue(highlighted);
        }
    }
}
