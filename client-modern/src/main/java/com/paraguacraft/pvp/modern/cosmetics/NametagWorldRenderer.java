package com.paraguacraft.pvp.modern.cosmetics;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.CullHelper;
import com.paraguacraft.pvp.modern.core.NickFinderManager;
import com.paraguacraft.pvp.modern.core.TeamColorHelper;
import com.paraguacraft.pvp.modern.network.BadgeProtocol;
import com.paraguacraft.pvp.modern.network.BadgeRegistry;
import com.paraguacraft.pvp.modern.network.ParaguacraftNetwork;
import net.fabricmc.fabric.api.client.rendering.v1.world.WorldRenderContext;
import net.fabricmc.fabric.api.client.rendering.v1.world.WorldRenderEvents;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.font.TextRenderer;
import net.minecraft.client.network.PlayerListEntry;
import net.minecraft.client.option.Perspective;
import net.minecraft.client.render.Camera;
import net.minecraft.client.render.LightmapTextureManager;
import net.minecraft.client.render.RenderLayers;
import net.minecraft.client.render.VertexConsumer;
import net.minecraft.client.render.VertexConsumerProvider;
import net.minecraft.client.render.state.CameraRenderState;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.entity.player.PlayerEntity;
import net.minecraft.text.Text;
import net.minecraft.text.TextColor;
import net.minecraft.util.math.RotationAxis;
import net.minecraft.util.math.Vec3d;
import org.joml.Matrix4f;

/**
 * Nametags 3D fuera del pipeline de EntityRenderer (Sodium-safe).
 * Billboard con yaw/pitch de cámara, pitch invertido en F5 frontal.
 */
public final class NametagWorldRenderer {

    private static final float LABEL_SCALE = 0.026666668F;

    private NametagWorldRenderer() {}

    public static void register() {
        WorldRenderEvents.AFTER_ENTITIES.register(NametagWorldRenderer::render);
    }

    private static void render(WorldRenderContext context) {
        if (!NametagOverlay.shouldReplace3dLabel()) {
            return;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.world == null || client.player == null || context.matrices() == null || context.consumers() == null) {
            return;
        }
        if (NametagOverlay.isGuiEntityPass()) {
            return;
        }
        CameraRenderState camera = context.worldState().cameraRenderState;
        if (camera == null || !camera.initialized) {
            return;
        }
        ParaguacraftNetwork.tickLocal();
        float tickDelta = client.getRenderTickCounter().getTickProgress(false);
        Vec3d cam = camera.pos;
        MatrixStack matrices = context.matrices();
        VertexConsumerProvider consumers = context.consumers();
        TextRenderer tr = client.textRenderer;
        int light = LightmapTextureManager.pack(15, 15);
        Perspective perspective = client.options.getPerspective();

        for (PlayerEntity player : client.world.getPlayers()) {
            boolean local = player == client.player;
            if (local && perspective.isFirstPerson()) {
                continue;
            }
            if (local && NametagOverlay.isLocalPreviewScreen(client.currentScreen)) {
                continue;
            }
            double distSq = client.player.squaredDistanceTo(player);
            if (ModernConfig.nametagCull && !local && distSq > CullHelper.NAMETAG_CULL_DISTANCE_SQ) {
                continue;
            }
            if (ModernConfig.nametagLod && !local && distSq > CullHelper.NAMETAG_LOD_DISTANCE_SQ) {
                if (client.targetedEntity != player) {
                    continue;
                }
            }
            drawPlayer(player, client, matrices, consumers, tr, cam, tickDelta, light, perspective);
        }
    }

    private static void drawPlayer(
        PlayerEntity player,
        MinecraftClient client,
        MatrixStack matrices,
        VertexConsumerProvider consumers,
        TextRenderer tr,
        Vec3d cam,
        float tickDelta,
        int light,
        Perspective perspective
    ) {
        Text name = resolveName(player, client);
        boolean local = player == client.player;
        boolean logo = local
            ? ModernConfig.showNametagLogo && ParaguacraftNetwork.hasLogo(player)
            : (ModernConfig.showNametagLogoOthers && ParaguacraftNetwork.hasLogo(player));

        double x = player.lastX + (player.getX() - player.lastX) * tickDelta - cam.x;
        double y = player.lastY + (player.getY() - player.lastY) * tickDelta - cam.y + player.getHeight() + 0.5;
        double z = player.lastZ + (player.getZ() - player.lastZ) * tickDelta - cam.z;

        Camera camera = client.gameRenderer.getCamera();
        float pitchSign = perspective.isFrontView() ? -1.0F : 1.0F;

        matrices.push();
        matrices.translate(x, y, z);
        matrices.multiply(RotationAxis.POSITIVE_Y.rotationDegrees(-camera.getYaw()));
        matrices.multiply(RotationAxis.POSITIVE_X.rotationDegrees(camera.getPitch() * pitchSign));
        matrices.scale(-LABEL_SCALE, -LABEL_SCALE, LABEL_SCALE);
        Matrix4f matrix = matrices.peek().getPositionMatrix();

        int nameW = tr.getWidth(name);
        float left = -nameW / 2.0F;
        if (logo) {
            left -= ParaguacraftTextures.MINI_SIZE + 2;
            drawIcon(consumers, matrix, left, -1, light);
            left += ParaguacraftTextures.MINI_SIZE + 2;
        }
        tr.draw(name, left, 0.0F, 0x20FFFFFF, false, matrix, consumers, TextRenderer.TextLayerType.SEE_THROUGH, 0x40000000, light);
        tr.draw(name, left, 0.0F, 0xFFFFFFFF, false, matrix, consumers, TextRenderer.TextLayerType.NORMAL, 0, light);

        if (ModernConfig.showNametagHealth) {
            String hp = String.format("%.1f", Math.max(0.0F, player.getHealth()));
            Text hpText = Text.literal(hp).styled(s -> s.withColor(TextColor.fromRgb(0xFF5555)));
            float hpW = tr.getWidth(hpText) / 2.0F;
            tr.draw(hpText, -hpW, 10.0F, 0xFFFF5555, false, matrix, consumers, TextRenderer.TextLayerType.NORMAL, 0, light);
        }
        matrices.pop();
    }

    private static Text resolveName(PlayerEntity player, MinecraftClient client) {
        Text source = player.getDisplayName();
        if (ModernConfig.nickFinderEnabled && NickFinderManager.isActive()) {
            Text highlighted = NickFinderManager.highlightLabel(source);
            if (highlighted != source) {
                source = highlighted;
            }
        } else if (ModernConfig.teamColors) {
            int rgb = TeamColorHelper.getNametagColor(player);
            if (rgb != -1) {
                source = Text.literal(source.getString()).styled(s -> s.withColor(TextColor.fromRgb(rgb & 0xFFFFFF)));
            }
        }
        boolean showBadge = player.equals(client.player) ? ModernConfig.showNametagLogo : ModernConfig.showNametagLogoOthers;
        if (showBadge && BadgeRegistry.hasBadge(player.getUuid())) {
            byte badge = BadgeRegistry.getBadge(player.getUuid());
            if (badge != BadgeProtocol.BADGE_NONE) {
                int color = badge == BadgeProtocol.BADGE_STAFF ? 0xFFD966 : 0x55E5FF;
                source = Text.literal("").append(
                    Text.literal("\u2605 ").styled(s -> s.withColor(TextColor.fromRgb(color)))
                ).append(source);
            }
        }
        if (!player.equals(client.player) && ModernConfig.showOpponentPing && client.getNetworkHandler() != null) {
            PlayerListEntry entry = client.getNetworkHandler().getPlayerListEntry(player.getUuid());
            if (entry != null && entry.getLatency() >= 0) {
                source = Text.literal("").append(source).append(
                    Text.literal(" " + entry.getLatency() + "ms").styled(s -> s.withColor(TextColor.fromRgb(0xAAAAAA)))
                );
            }
        }
        return source;
    }

    private static void drawIcon(VertexConsumerProvider consumers, Matrix4f matrix, float x, float y, int light) {
        float s = ParaguacraftTextures.MINI_SIZE;
        VertexConsumer see = consumers.getBuffer(RenderLayers.textSeeThrough(ParaguacraftTextures.MINI_ICON));
        quad(see, matrix, x, y, s, light, 180);
        VertexConsumer solid = consumers.getBuffer(RenderLayers.text(ParaguacraftTextures.MINI_ICON));
        quad(solid, matrix, x, y, s, light, 255);
    }

    private static void quad(VertexConsumer vc, Matrix4f matrix, float x, float y, float s, int light, int alpha) {
        vc.vertex(matrix, x, y + s, 0).color(255, 255, 255, alpha).texture(0f, 1f).light(light);
        vc.vertex(matrix, x + s, y + s, 0).color(255, 255, 255, alpha).texture(1f, 1f).light(light);
        vc.vertex(matrix, x + s, y, 0).color(255, 255, 255, alpha).texture(1f, 0f).light(light);
        vc.vertex(matrix, x, y, 0).color(255, 255, 255, alpha).texture(0f, 0f).light(light);
    }
}
