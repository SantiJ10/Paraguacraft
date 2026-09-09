package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.hud.PlayerListHud;
import net.minecraft.client.network.PlayerListEntry;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.ArrayList;
import java.util.List;
import java.util.UUID;

@Mixin(PlayerListHud.class)
public class MixinPlayerListHudEditor {

    @Inject(method = "collectPlayerEntries", at = @At("RETURN"), cancellable = true, require = 0)
    private void paraguacraft$hideNpcs(CallbackInfoReturnable<List<PlayerListEntry>> cir) {
        if (!ModernConfig.tabEditor || !ModernConfig.tabHideNpcs || cir.getReturnValue() == null) {
            return;
        }
        List<PlayerListEntry> out = new ArrayList<>();
        for (PlayerListEntry entry : cir.getReturnValue()) {
            if (!isNpc(entry)) {
                out.add(entry);
            }
        }
        cir.setReturnValue(out);
    }

    @Inject(method = "renderLatencyIcon", at = @At("HEAD"), cancellable = true, require = 0)
    private void paraguacraft$pingNumbers(DrawContext context, int width, int x, int y, PlayerListEntry entry, CallbackInfo ci) {
        if (!ModernConfig.tabEditor) {
            return;
        }
        int ping = entry.getLatency();
        if (ModernConfig.tabHideHighPing && ping > 500) {
            ci.cancel();
            return;
        }
        if (!ModernConfig.tabPingNumbers) {
            return;
        }
        ci.cancel();
        String text = String.valueOf(ping);
        int color = ping < 80 ? 0xFF55FF55 : (ping < 150 ? 0xFFFFFF55 : 0xFFFF5555);
        int tx = x + width - MinecraftClient.getInstance().textRenderer.getWidth(text) - 2;
        context.drawTextWithShadow(MinecraftClient.getInstance().textRenderer, text, tx, y, color);
    }

    private static boolean isNpc(PlayerListEntry entry) {
        if (entry == null || entry.getProfile() == null) {
            return false;
        }
        UUID id = entry.getProfile().id();
        if (id != null && id.version() == 2) {
            return true;
        }
        String name = entry.getProfile().name();
        if (name == null) {
            return false;
        }
        String lower = name.toLowerCase();
        return lower.contains("npc") || lower.startsWith("cit-");
    }
}
