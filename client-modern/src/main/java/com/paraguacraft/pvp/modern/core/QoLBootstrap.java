package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.mixin.KeyBindingAccessor;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientEntityEvents;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.message.v1.ClientReceiveMessageEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.fabricmc.fabric.api.event.player.AttackEntityCallback;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import net.minecraft.util.ActionResult;

/** Registra eventos QoL: fullbright, combo, chat triggers y toggle sneak. */
public final class QoLBootstrap {

    private static float lastHealth = -1F;
    private static boolean sneakKeyWasPressed = false;
    private static boolean wasDead = false;

    private QoLBootstrap() {}

    public static void register() {
        ClientTickEvents.END_CLIENT_TICK.register(QoLBootstrap::tick);
        ClientReceiveMessageEvents.GAME.register(ChatTriggerManager::onChatMessage);
        AttackEntityCallback.EVENT.register((player, world, hand, entity, hitResult) -> {
            CombatStats.onAttack(entity);
            return ActionResult.PASS;
        });
        ClientEntityEvents.ENTITY_UNLOAD.register((entity, world) -> CombatStats.onEntityUnload(entity));
        ClientPlayConnectionEvents.JOIN.register((handler, sender, client) -> {
            CombatStats.reset();
            lastHealth = -1F;
            wasDead = false;
        });
    }

    private static void tick(MinecraftClient client) {
        CombatStats.pruneRecentHits();
        if (client.player == null) {
            return;
        }
        float hp = client.player.getHealth();
        if (lastHealth >= 0 && hp < lastHealth) {
            CombatStats.onPlayerHurt();
        }
        boolean dead = client.player.isDead();
        if (dead && !wasDead) {
            CombatStats.onOwnDeath();
        }
        wasDead = dead;
        lastHealth = hp;
        tickToggleSneak(client);
    }

    /** Toggle sneak (paridad 1.8.9 `QoLManager`): flanco de bajada de Shift alterna sneak permanente. */
    private static void tickToggleSneak(MinecraftClient client) {
        if (!ModernConfig.toggleSneak) {
            if (ModernConfig.isSneakingToggled) {
                ModernConfig.isSneakingToggled = false;
                sneakKeyWasPressed = false;
            }
            return;
        }
        if (client.currentScreen != null) {
            return;
        }
        KeyBinding sneakKey = client.options.sneakKey;
        InputUtil.Key bound = ((KeyBindingAccessor) sneakKey).paraguacraft$getBoundKey();
        boolean physical = InputUtil.isKeyPressed(client.getWindow(), bound.getCode());
        if (physical && !sneakKeyWasPressed) {
            ModernConfig.isSneakingToggled = !ModernConfig.isSneakingToggled;
        }
        sneakKeyWasPressed = physical;
    }
}

