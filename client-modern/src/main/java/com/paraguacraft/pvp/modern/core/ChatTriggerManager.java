package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.sound.SoundEvents;
import net.minecraft.text.Text;
import net.minecraft.util.Formatting;

import java.util.Locale;

/** Alertas en chat por palabras clave (BedWars / Hypixel). */
public final class ChatTriggerManager {

    private static final String[][] RULES = {
        {"bed", "¡CAMA!", Formatting.RED.getName()},
        {"final kill", "FINAL KILL", Formatting.GOLD.getName()},
        {"trapped", "TRAMPA", Formatting.DARK_RED.getName()},
        {"bridge", "BRIDGE", Formatting.AQUA.getName()},
    };

    private ChatTriggerManager() {}

    public static void onChatMessage(Text message, boolean overlay) {
        if (overlay || !ModernConfig.chatTriggers || message == null) {
            return;
        }
        String plain = ScoreboardFilter.strip(message).toLowerCase(Locale.ROOT);
        if (plain.isEmpty()) {
            return;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        for (String[] rule : RULES) {
            if (plain.contains(rule[0])) {
                Formatting fmt = parseColor(rule[2]);
                client.inGameHud.setTitle(Text.literal(rule[1]).formatted(fmt, Formatting.BOLD));
                client.inGameHud.setSubtitle(Text.empty());
                client.inGameHud.setTitleTicks(5, 30, 10);
                if (client.player != null) {
                    client.player.playSound(SoundEvents.BLOCK_NOTE_BLOCK_PLING.value(), 1.0F, 1.2F);
                }
                return;
            }
        }
    }

    private static Formatting parseColor(String name) {
        if (name == null) {
            return Formatting.RED;
        }
        try {
            return Formatting.valueOf(name.toUpperCase(Locale.ROOT));
        } catch (IllegalArgumentException e) {
            return Formatting.RED;
        }
    }
}
