package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.NickFinderManager;
import com.paraguacraft.pvp.modern.core.TeamColorHelper;
import com.paraguacraft.pvp.modern.network.BadgeProtocol;
import com.paraguacraft.pvp.modern.network.BadgeRegistry;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.PlayerListEntry;
import net.minecraft.client.render.command.OrderedRenderCommandQueue;
import net.minecraft.client.render.entity.EntityRenderer;
import net.minecraft.client.render.entity.state.EntityRenderState;
import net.minecraft.client.render.entity.state.PlayerEntityRenderState;
import net.minecraft.client.render.state.CameraRenderState;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.entity.Entity;
import net.minecraft.entity.player.PlayerEntity;
import net.minecraft.text.Text;
import net.minecraft.text.TextColor;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.UUID;

/** Resalta nicks encontrados en nametags 3D (estilo Hytils NickFinder) + insignias/ping (Fase 3). */
@Mixin(EntityRenderer.class)
public abstract class MixinEntityRendererNickfinder<T extends Entity, S extends EntityRenderState> {

    @Inject(
        method = "renderLabelIfPresent",
        at = @At("HEAD")
    )
    private void paraguacraft$nickNametag(
        S state,
        MatrixStack matrices,
        OrderedRenderCommandQueue queue,
        CameraRenderState cameraRenderState,
        CallbackInfo ci
    ) {
        if (state == null) {
            return;
        }
        Text source = resolveName(state);
        if (source == null) {
            return;
        }

        Text styled = source;
        if (ModernConfig.nickFinderEnabled && NickFinderManager.isActive()) {
            Text highlighted = NickFinderManager.highlightLabel(source);
            if (highlighted != source) {
                styled = highlighted;
            }
        } else if (ModernConfig.teamColors && state instanceof PlayerEntityRenderState playerState) {
            styled = applyTeamColor(playerState, source);
        }

        if (state instanceof PlayerEntityRenderState playerState) {
            styled = appendBadgeAndPing(playerState, styled);
        }

        if (styled != source) {
            state.displayName = styled;
            if (state instanceof PlayerEntityRenderState playerState) {
                playerState.playerName = styled;
            }
        }
    }

    /**
     * Insignia Paraguacraft/Staff + ping rival (paridad con {@code MixinNametagLogo} 1.8.9).
     * Se antepone/pospone como texto plano ya que el pipeline de render moderno no admite
     * dibujar texturas extra en {@code renderLabelIfPresent} de forma segura.
     */
    private static Text appendBadgeAndPing(PlayerEntityRenderState playerState, Text name) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.world == null) {
            return name;
        }
        Entity entity = client.world.getEntityById(playerState.id);
        if (!(entity instanceof PlayerEntity player)) {
            return name;
        }
        UUID id = player.getUuid();
        boolean isLocal = client.player != null && id.equals(client.player.getUuid());

        Text result = name;
        boolean showBadge = isLocal ? ModernConfig.showNametagLogo : ModernConfig.showNametagLogoOthers;
        if (showBadge && BadgeRegistry.hasBadge(id)) {
            byte badge = BadgeRegistry.getBadge(id);
            if (badge != BadgeProtocol.BADGE_NONE) {
                int color = badge == BadgeProtocol.BADGE_STAFF ? 0xFFD966 : 0x55E5FF;
                Text icon = Text.literal("\u2605 ").styled(s -> s.withColor(TextColor.fromRgb(color)));
                result = Text.literal("").append(icon).append(result);
            }
        }

        if (!isLocal && ModernConfig.showOpponentPing && client.getNetworkHandler() != null) {
            PlayerListEntry entry = client.getNetworkHandler().getPlayerListEntry(id);
            if (entry != null && entry.getLatency() >= 0) {
                Text ping = Text.literal(" " + entry.getLatency() + "ms")
                    .styled(s -> s.withColor(TextColor.fromRgb(0xAAAAAA)));
                result = Text.literal("").append(result).append(ping);
            }
        }
        return result;
    }

    private static Text resolveName(EntityRenderState state) {
        if (state instanceof PlayerEntityRenderState playerState && playerState.playerName != null) {
            return playerState.playerName;
        }
        return state.displayName;
    }

    private static Text applyTeamColor(PlayerEntityRenderState playerState, Text source) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.world == null) {
            return source;
        }
        Entity entity = client.world.getEntityById(playerState.id);
        if (!(entity instanceof PlayerEntity player)) {
            return source;
        }
        int rgb = TeamColorHelper.getNametagColor(player);
        if (rgb == -1) {
            return source;
        }
        String plain = source.getString();
        return Text.literal(plain).styled(s -> s.withColor(TextColor.fromRgb(rgb & 0xFFFFFF)));
    }
}
