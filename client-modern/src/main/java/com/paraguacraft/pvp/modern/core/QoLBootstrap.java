package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.message.v1.ClientReceiveMessageEvents;
import net.fabricmc.fabric.api.event.player.AttackEntityCallback;
import net.minecraft.client.MinecraftClient;
import net.minecraft.util.ActionResult;

/** Registra eventos QoL: fullbright, combo y chat triggers. */
public final class QoLBootstrap {

    private static float lastHealth = -1F;

    private QoLBootstrap() {}

    public static void register() {
        ClientTickEvents.END_CLIENT_TICK.register(QoLBootstrap::tick);
        ClientReceiveMessageEvents.GAME.register(ChatTriggerManager::onChatMessage);
        AttackEntityCallback.EVENT.register((player, world, hand, entity, hitResult) -> {
            CombatStats.onAttack(entity);
            return ActionResult.PASS;
        });
    }

    private static void tick(MinecraftClient client) {
        if (client.player == null) {
            return;
        }
        if (ModernConfig.fullbright) {
            client.options.getGamma().setValue(16.0);
        }
        float hp = client.player.getHealth();
        if (lastHealth >= 0 && hp < lastHealth) {
            CombatStats.onPlayerHurt();
        }
        lastHealth = hp;
    }
}
