package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.sound.SoundEvents;
import net.minecraft.text.Text;
import net.minecraft.util.Formatting;

import java.util.List;
import java.util.Locale;

/** Alertas en chat por palabras clave (BedWars / Hypixel). */
public final class ChatTriggerManager {

    private ChatTriggerManager() {}

    public static void onChatMessage(Text message, boolean overlay) {
        if (overlay || !ModernConfig.chatTriggers || message == null) {
            return;
        }
        String plain = ScoreboardFilter.strip(message).toLowerCase(Locale.ROOT);
        if (plain.isEmpty()) {
            return;
        }
        ChatTriggerConfig.ensureLoaded();
        MinecraftClient client = MinecraftClient.getInstance();
        for (ChatTriggerConfig.Rule rule : ChatTriggerConfig.getRules()) {
            if (rule == null || !rule.enabled) {
                continue;
            }
            if (rule.hypixelOnly && !HypixelHelper.isOnHypixel(client)) {
                continue;
            }
            for (String keyword : ChatTriggerConfig.keywords(rule)) {
                if (plain.contains(keyword)) {
                    Formatting fmt = parseColor(rule.color);
                    client.inGameHud.setTitle(Text.literal(rule.title).formatted(fmt, Formatting.BOLD));
                    client.inGameHud.setSubtitle(Text.empty());
                    client.inGameHud.setTitleTicks(rule.fadeIn, rule.stay, rule.fadeOut);
                    if (client.player != null) {
                        client.player.playSound(SoundEvents.BLOCK_NOTE_BLOCK_PLING.value(), 1.0F, 1.2F);
                    }
                    return;
                }
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
