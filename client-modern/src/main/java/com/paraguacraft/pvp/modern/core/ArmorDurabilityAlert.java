package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.minecraft.client.MinecraftClient;
import net.minecraft.entity.EquipmentSlot;
import net.minecraft.item.ItemStack;
import net.minecraft.text.Text;
import net.minecraft.util.Formatting;

public final class ArmorDurabilityAlert {

    private static final EquipmentSlot[] SLOTS = {
        EquipmentSlot.FEET, EquipmentSlot.LEGS, EquipmentSlot.CHEST, EquipmentSlot.HEAD
    };
    private static final String[] NAMES = {"Botas", "Pantalones", "Peto", "Casco"};
    private static final long COOLDOWN_MS = 8000L;
    private static final long[] LAST = new long[4];

    private ArmorDurabilityAlert() {}

    public static void register() {
        ClientTickEvents.END_CLIENT_TICK.register(ArmorDurabilityAlert::tick);
    }

    private static void tick(MinecraftClient client) {
        if (!ModernConfig.armorDurabilityAlert || client.player == null) {
            return;
        }
        long now = System.currentTimeMillis();
        for (int i = 0; i < SLOTS.length; i++) {
            ItemStack stack = client.player.getEquippedStack(SLOTS[i]);
            if (stack.isEmpty() || !stack.isDamageable()) {
                continue;
            }
            int max = stack.getMaxDamage();
            if (max <= 0) {
                continue;
            }
            int percent = (int) (((max - stack.getDamage()) * 100.0F) / max);
            if (percent >= 25) {
                continue;
            }
            if (now - LAST[i] < COOLDOWN_MS) {
                continue;
            }
            LAST[i] = now;
            client.player.sendMessage(
                Text.literal("[Armadura] " + NAMES[i] + " al " + percent + "%").formatted(Formatting.RED),
                false
            );
        }
    }
}
