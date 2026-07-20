package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.NickFinderManager;
import com.paraguacraft.pvp.modern.core.TeamColorHelper;
import net.minecraft.client.MinecraftClient;
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

/** Resalta nicks encontrados en nametags 3D (estilo Hytils NickFinder). */
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

        if (styled != source) {
            state.displayName = styled;
            if (state instanceof PlayerEntityRenderState playerState) {
                playerState.playerName = styled;
            }
        }
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
