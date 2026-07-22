package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.item.BlockItem;
import net.minecraft.item.ItemStack;

/** Cronometro visual mientras bridgeas (bloques en mano + sin suelo). */
public final class BridgeTimer {

    private static long bridgeStartMs;
    private static boolean bridging;

    private BridgeTimer() {}

    public static void tick(MinecraftClient client) {
        if (!ModernConfig.showBridgeTimer || client == null || client.player == null) {
            bridging = false;
            return;
        }
        if (!GameModeDetector.inMatch()
            && GameModeDetector.current() != GameModeDetector.Mode.PVP) {
            bridging = false;
            return;
        }
        boolean holdingBlocks = false;
        ItemStack main = client.player.getMainHandStack();
        ItemStack off = client.player.getOffHandStack();
        if (!main.isEmpty() && main.getItem() instanceof BlockItem) {
            holdingBlocks = true;
        }
        if (!off.isEmpty() && off.getItem() instanceof BlockItem) {
            holdingBlocks = true;
        }
        boolean midair = !client.player.isOnGround() && !client.player.isTouchingWater();
        if (holdingBlocks && midair) {
            if (!bridging) {
                bridgeStartMs = System.currentTimeMillis();
                bridging = true;
            }
        } else if (client.player.isOnGround()) {
            bridging = false;
        }
    }

    public static boolean active() {
        return bridging;
    }

    public static float seconds() {
        if (!bridging) {
            return 0F;
        }
        return (System.currentTimeMillis() - bridgeStartMs) / 1000F;
    }
}
