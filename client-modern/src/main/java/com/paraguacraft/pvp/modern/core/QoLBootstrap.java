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
import net.minecraft.sound.SoundEvents;
import net.minecraft.text.Text;
import net.minecraft.util.ActionResult;
import net.minecraft.util.Formatting;

import java.util.Locale;

/** Registra eventos QoL: fullbright, combo, chat triggers y toggle sneak. */
public final class QoLBootstrap {

    private static float lastHealth = -1F;
    private static boolean sneakKeyWasPressed = false;
    private static boolean wasDead = false;
    /** Evita recursión: sendMessage() vuelve a disparar GAME. */
    private static final ThreadLocal<Boolean> HANDLING_GAME_MESSAGE =
        ThreadLocal.withInitial(() -> false);

    private QoLBootstrap() {}

    public static void register() {
        ClientTickEvents.END_CLIENT_TICK.register(QoLBootstrap::tick);
        ClientReceiveMessageEvents.GAME.register(QoLBootstrap::onChatMessage);
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

    private static void onChatMessage(Text message, boolean overlay) {
        if (HANDLING_GAME_MESSAGE.get()) {
            return;
        }
        HANDLING_GAME_MESSAGE.set(true);
        try {
            ChatTriggerManager.onChatMessage(message, overlay);
            if (overlay || !ModernConfig.chatAlertsEnabled || message == null) {
                return;
            }
            String plain = ScoreboardFilter.strip(message).toLowerCase(Locale.ROOT);
            String match = ChatAlerts.firstMatch(plain);
            if (match == null) {
                return;
            }
            MinecraftClient client = MinecraftClient.getInstance();
            if (ChatAlerts.highlight && client.player != null) {
                client.player.sendMessage(
                    Text.literal("[Alerta] ").formatted(Formatting.GOLD)
                        .append(Text.literal(match).formatted(ChatAlerts.colorFmt(), Formatting.BOLD)),
                    false
                );
            }
            if (ChatAlerts.sound && client.player != null) {
                client.player.playSound(SoundEvents.BLOCK_NOTE_BLOCK_BELL.value(), 0.8F, 1.4F);
            }
        } finally {
            HANDLING_GAME_MESSAGE.set(false);
        }
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

