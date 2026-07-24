package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.fabricmc.fabric.api.client.rendering.v1.world.WorldRenderContext;
import net.fabricmc.fabric.api.client.rendering.v1.world.WorldRenderEvents;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.font.TextRenderer;
import net.minecraft.client.render.LightmapTextureManager;
import net.minecraft.client.render.state.CameraRenderState;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.entity.Entity;
import net.minecraft.entity.TntEntity;
import net.minecraft.text.Text;
import net.minecraft.util.math.Vec3d;
import org.joml.Matrix4f;

/**
 * Contador estilo Lunar sobre TNT encendida (ej. {@code 1,35}).
 * WorldRenderEvents.AFTER_ENTITIES — independiente del nametag vanilla.
 */
public final class TntCountdownBootstrap {

    private TntCountdownBootstrap() {}

    public static void register() {
        WorldRenderEvents.AFTER_ENTITIES.register(TntCountdownBootstrap::render);
    }

    private static void render(WorldRenderContext context) {
        if (!ModernConfig.showTntCountdown) {
            return;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.world == null || context.matrices() == null || context.consumers() == null) {
            return;
        }

        CameraRenderState camera = context.worldState().cameraRenderState;
        if (camera == null || !camera.initialized) {
            return;
        }

        TextRenderer textRenderer = client.textRenderer;
        Vec3d cam = camera.pos;
        float tickDelta = client.getRenderTickCounter().getTickProgress(false);
        MatrixStack matrices = context.matrices();

        for (Entity entity : client.world.getEntities()) {
            if (!(entity instanceof TntEntity tnt)) {
                continue;
            }
            int fuse = tnt.getFuse();
            if (fuse <= 0) {
                continue;
            }

            float fuseLeft = Math.max(0.0F, fuse - tickDelta);
            String label = String.format(java.util.Locale.US, "%.2f", fuseLeft / 20.0F).replace('.', ',');
            Text text = Text.literal(label);

            double x = tnt.lastX + (tnt.getX() - tnt.lastX) * tickDelta - cam.x;
            double y = tnt.lastY + (tnt.getY() - tnt.lastY) * tickDelta - cam.y + tnt.getHeight() + 0.35;
            double z = tnt.lastZ + (tnt.getZ() - tnt.lastZ) * tickDelta - cam.z;

            matrices.push();
            matrices.translate(x, y, z);
            matrices.multiply(camera.orientation);
            matrices.scale(0.025F, -0.025F, 0.025F);

            Matrix4f matrix = matrices.peek().getPositionMatrix();
            float width = textRenderer.getWidth(text) / 2.0F;
            int light = LightmapTextureManager.pack(15, 15);

            textRenderer.draw(
                text,
                -width,
                0.0F,
                0x20FFFFFF,
                false,
                matrix,
                context.consumers(),
                TextRenderer.TextLayerType.SEE_THROUGH,
                0x40000000,
                light
            );
            textRenderer.draw(
                text,
                -width,
                0.0F,
                0xFFFFFFFF,
                false,
                matrix,
                context.consumers(),
                TextRenderer.TextLayerType.NORMAL,
                0,
                light
            );

            matrices.pop();
        }
    }
}
