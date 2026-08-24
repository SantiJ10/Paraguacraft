package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.fabricmc.fabric.api.client.rendering.v1.world.WorldRenderContext;
import net.fabricmc.fabric.api.client.rendering.v1.world.WorldRenderEvents;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.font.TextRenderer;
import net.minecraft.client.render.LightmapTextureManager;
import net.minecraft.client.render.state.CameraRenderState;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.text.Text;
import net.minecraft.util.math.Vec3d;
import org.joml.Matrix4f;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Locale;

public final class WaypointManager {

    public record Waypoint(String name, int dim, double x, double y, double z) {}

    private static final List<Waypoint> POINTS = new ArrayList<>();
    private static boolean loaded;

    private WaypointManager() {}

    public static void register() {
        WorldRenderEvents.AFTER_ENTITIES.register(WaypointManager::renderWorld);
    }

    public static List<Waypoint> all() {
        ensureLoaded();
        return POINTS;
    }

    public static void add(String name, int dim, double x, double y, double z) {
        ensureLoaded();
        remove(name);
        POINTS.add(new Waypoint(name, dim, x, y, z));
        save();
    }

    public static boolean remove(String name) {
        ensureLoaded();
        boolean found = false;
        Iterator<Waypoint> it = POINTS.iterator();
        while (it.hasNext()) {
            if (it.next().name.equalsIgnoreCase(name)) {
                it.remove();
                found = true;
            }
        }
        if (found) {
            save();
        }
        return found;
    }

    public static void drawHud(net.minecraft.client.gui.DrawContext ctx, TextRenderer tr) {
        if (!ModernConfig.showWaypoints) {
            return;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.player == null || client.world == null) {
            return;
        }
        ensureLoaded();
        int dim = client.world.getRegistryKey().getValue().toString().hashCode();
        int y = 0;
        int shown = 0;
        for (Waypoint wp : POINTS) {
            if (wp.dim != dim) {
                continue;
            }
            double dx = wp.x - client.player.getX();
            double dy = wp.y - client.player.getY();
            double dz = wp.z - client.player.getZ();
            int dist = (int) Math.sqrt(dx * dx + dy * dy + dz * dz);
            ctx.drawText(tr, Text.literal(wp.name + " " + dist + "m"), 0, y, 0xFF00E5FF, true);
            y += 10;
            if (++shown >= 8) {
                break;
            }
        }
    }

    public static int hudHeight() {
        return 80;
    }

    private static void renderWorld(WorldRenderContext context) {
        if (!ModernConfig.showWaypoints || context.matrices() == null || context.consumers() == null) {
            return;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.world == null || client.player == null) {
            return;
        }
        ensureLoaded();
        CameraRenderState camera = context.worldState().cameraRenderState;
        if (camera == null || !camera.initialized) {
            return;
        }
        int dim = client.world.getRegistryKey().getValue().toString().hashCode();
        float tickDelta = client.getRenderTickCounter().getTickProgress(false);
        Vec3d cam = camera.pos;
        MatrixStack matrices = context.matrices();
        TextRenderer tr = client.textRenderer;
        int light = LightmapTextureManager.pack(15, 15);
        for (Waypoint wp : POINTS) {
            if (wp.dim != dim) {
                continue;
            }
            double x = wp.x - cam.x;
            double y = wp.y - cam.y + 1.8;
            double z = wp.z - cam.z;
            double dist = Math.sqrt(x * x + y * y + z * z);
            if (dist > 256.0) {
                continue;
            }
            Text label = Text.literal(wp.name + " " + (int) dist + "m");
            matrices.push();
            matrices.translate(x, y, z);
            matrices.multiply(camera.orientation);
            matrices.scale(0.025F, -0.025F, 0.025F);
            Matrix4f matrix = matrices.peek().getPositionMatrix();
            float w = tr.getWidth(label) / 2.0F;
            tr.draw(label, -w, 0.0F, 0xFF00E5FF, false, matrix, context.consumers(), TextRenderer.TextLayerType.NORMAL, 0x40000000, light);
            matrices.pop();
        }
    }

    private static void ensureLoaded() {
        if (loaded) {
            return;
        }
        loaded = true;
        POINTS.clear();
        Path file = file();
        if (!Files.isRegularFile(file)) {
            return;
        }
        try {
            for (String line : Files.readAllLines(file, StandardCharsets.UTF_8)) {
                String[] p = line.split("\\|", -1);
                if (p.length < 5) {
                    continue;
                }
                POINTS.add(new Waypoint(p[0], Integer.parseInt(p[1]), Double.parseDouble(p[2]), Double.parseDouble(p[3]), Double.parseDouble(p[4])));
            }
        } catch (Exception ignored) {
        }
    }

    private static void save() {
        Path file = file();
        try {
            Files.createDirectories(file.getParent());
            StringBuilder sb = new StringBuilder();
            for (Waypoint wp : POINTS) {
                sb.append(wp.name).append('|').append(wp.dim).append('|')
                    .append(String.format(Locale.US, "%.2f|%.2f|%.2f", wp.x, wp.y, wp.z))
                    .append('\n');
            }
            Files.writeString(file, sb.toString(), StandardCharsets.UTF_8);
        } catch (IOException ignored) {
        }
    }

    private static Path file() {
        return net.fabricmc.loader.api.FabricLoader.getInstance().getGameDir().resolve("config/paraguacraft-waypoints.txt");
    }
}
