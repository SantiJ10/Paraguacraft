package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.fabricmc.fabric.api.client.rendering.v1.world.WorldRenderContext;
import net.fabricmc.fabric.api.client.rendering.v1.world.WorldRenderEvents;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.font.TextRenderer;
import net.minecraft.client.network.PlayerListEntry;
import net.minecraft.client.render.LightmapTextureManager;
import net.minecraft.client.render.state.CameraRenderState;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.entity.Entity;
import net.minecraft.entity.TntEntity;
import net.minecraft.text.Text;
import net.minecraft.util.math.Vec3d;
import org.joml.Matrix4f;

import java.util.HashMap;
import java.util.HashSet;
import java.util.Iterator;
import java.util.Map;
import java.util.Set;

/**
 * Contador de TNT sincronizado con reloj de pared, compensación de ping
 * y fuse override (52 ticks en Minemen practice).
 */
public final class TntCountdownBootstrap {

    private static final class Track {
        long seenNanos;
        int startFuse;
    }

    private static final Map<Integer, Track> TRACKS = new HashMap<>();

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
        Set<Integer> seen = new HashSet<>();

        for (Entity entity : client.world.getEntities()) {
            if (!(entity instanceof TntEntity tnt)) {
                continue;
            }
            seen.add(tnt.getId());
            float fuseLeft = remainingTicks(client, tnt);
            if (fuseLeft <= 0.0F) {
                continue;
            }

            String label = String.format(java.util.Locale.US, "%.2f", fuseLeft / 20.0F).replace('.', ',');
            Text text = Text.literal(label);
            int rgb = fuseColor(fuseLeft);

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
                text, -width, 0.0F, 0x20000000 | (rgb & 0xFFFFFF), false, matrix,
                context.consumers(), TextRenderer.TextLayerType.SEE_THROUGH, 0x40000000, light
            );
            textRenderer.draw(
                text, -width, 0.0F, 0xFF000000 | (rgb & 0xFFFFFF), false, matrix,
                context.consumers(), TextRenderer.TextLayerType.NORMAL, 0, light
            );
            matrices.pop();
        }
        prune(seen);
    }

    private static float remainingTicks(MinecraftClient client, TntEntity tnt) {
        int id = tnt.getId();
        Track track = TRACKS.get(id);
        if (track == null) {
            track = new Track();
            long pingOffset = 0L;
            if (ModernConfig.tntPingCompensate) {
                pingOffset = (long) (pingMs(client) * 500_000.0D);
            }
            track.seenNanos = System.nanoTime() - pingOffset;
            track.startFuse = Math.max(1, tnt.getFuse());
            TRACKS.put(id, track);
        }
        int fuseCap = resolveFuse(client, track.startFuse);
        float elapsed = (System.nanoTime() - track.seenNanos) / 50_000_000.0F;
        return Math.max(0.0F, fuseCap - elapsed);
    }

    private static int resolveFuse(MinecraftClient client, int startFuse) {
        if (ModernConfig.tntFuseOverride > 0) {
            return ModernConfig.tntFuseOverride;
        }
        if (client.getCurrentServerEntry() != null && client.getCurrentServerEntry().address != null) {
            String ip = client.getCurrentServerEntry().address.toLowerCase();
            if (ip.contains("minemen") || ip.contains("mineman") || ip.contains("mmc.") || ip.contains("minemenclub")) {
                return 52;
            }
        }
        return startFuse;
    }

    private static float pingMs(MinecraftClient client) {
        if (client.player == null || client.getNetworkHandler() == null) {
            return 0.0F;
        }
        PlayerListEntry info = client.getNetworkHandler().getPlayerListEntry(client.player.getUuid());
        if (info == null) {
            return 0.0F;
        }
        return Math.max(0, info.getLatency());
    }

    private static int fuseColor(float fuseLeft) {
        float sec = fuseLeft / 20.0F;
        if (sec > 2.0F) {
            return 0x55FF55;
        }
        if (sec > 1.0F) {
            return 0xFFFF55;
        }
        return 0xFF5555;
    }

    private static void prune(Set<Integer> alive) {
        Iterator<Map.Entry<Integer, Track>> it = TRACKS.entrySet().iterator();
        while (it.hasNext()) {
            if (!alive.contains(it.next().getKey())) {
                it.remove();
            }
        }
    }
}
