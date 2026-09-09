package com.paraguacraft.pvp.mixins;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.GuiPlayerTabOverlay;
import net.minecraft.client.network.NetHandlerPlayClient;
import net.minecraft.client.network.NetworkPlayerInfo;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.UUID;

@Mixin(GuiPlayerTabOverlay.class)
public class MixinGuiPlayerTabOverlay {

    @Redirect(
        method = "renderPlayerlist",
        at = @At(value = "INVOKE", target = "Lnet/minecraft/client/network/NetHandlerPlayClient;getPlayerInfoMap()Ljava/util/Collection;")
    )
    private Collection<NetworkPlayerInfo> paraguacraft$filterTab(NetHandlerPlayClient net) {
        Collection<NetworkPlayerInfo> orig = net.getPlayerInfoMap();
        if (!ModConfig.tabEditor || !ModConfig.tabHideNpcs || orig == null) {
            return orig;
        }
        List<NetworkPlayerInfo> out = new ArrayList<NetworkPlayerInfo>();
        for (NetworkPlayerInfo info : orig) {
            if (!isNpc(info)) {
                out.add(info);
            }
        }
        return out;
    }

    @Inject(method = "drawPing", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$pingNumbers(int width, int x, int y, NetworkPlayerInfo info, CallbackInfo ci) {
        if (!ModConfig.tabEditor) {
            return;
        }
        int ping = info.getResponseTime();
        if (ModConfig.tabHideHighPing && ping > 500) {
            ci.cancel();
            return;
        }
        if (!ModConfig.tabPingNumbers) {
            return;
        }
        ci.cancel();
        String text = String.valueOf(ping);
        int color = ping < 80 ? 0xFF55FF55 : (ping < 150 ? 0xFFFFFF55 : 0xFFFF5555);
        FontRenderer fr = Minecraft.getMinecraft().fontRendererObj;
        int tx = x + width - fr.getStringWidth(text) - 2;
        fr.drawStringWithShadow(text, tx, y, color);
    }

    private static boolean isNpc(NetworkPlayerInfo info) {
        if (info == null || info.getGameProfile() == null) {
            return false;
        }
        UUID id = info.getGameProfile().getId();
        if (id != null && id.version() == 2) {
            return true;
        }
        String name = info.getGameProfile().getName();
        if (name == null) {
            return false;
        }
        String lower = name.toLowerCase();
        return lower.contains("npc") || lower.startsWith("cit-") || name.startsWith("§8");
    }
}
