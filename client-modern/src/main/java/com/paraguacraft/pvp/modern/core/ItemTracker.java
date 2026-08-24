package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.fabricmc.fabric.api.client.rendering.v1.world.WorldRenderContext;
import net.fabricmc.fabric.api.client.rendering.v1.world.WorldRenderEvents;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.font.TextRenderer;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.render.LightmapTextureManager;
import net.minecraft.client.render.state.CameraRenderState;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.entity.ItemEntity;
import net.minecraft.item.ItemStack;
import net.minecraft.text.Text;
import net.minecraft.util.math.Vec3d;
import org.joml.Matrix3x2fStack;
import org.joml.Matrix4f;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/** Rastrea items dropeados: lista 2D compacta y etiquetas 3D discretas. */
public final class ItemTracker {

    private static final double RANGE_2D = 16.0;
    private static final double RANGE_3D = 10.0;
    private static final int HUD_MAX = 5;
    private static final int WORLD_MAX = 4;
    private static final float HUD_SCALE = 0.72F;
    private static final int ROW = 10;
    private static final float LABEL_SCALE = 0.016F;

    private ItemTracker() {}

    public static void register() {
        WorldRenderEvents.AFTER_ENTITIES.register(ItemTracker::renderWorld);
    }

    public static void drawHud(DrawContext ctx, TextRenderer tr) {
        if (!ModernConfig.itemTracker2d) {
            return;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        List<ItemEntity> items = nearby(client, RANGE_2D);
        int n = Math.min(HUD_MAX, items.size());
        Matrix3x2fStack matrices = ctx.getMatrices();
        matrices.pushMatrix();
        matrices.scale(HUD_SCALE, HUD_SCALE);
        int y = 0;
        for (int i = 0; i < n; i++) {
            ItemEntity ei = items.get(i);
            ItemStack stack = ei.getStack();
            if (stack.isEmpty()) {
                continue;
            }
            int dist = (int) Math.ceil(client.player.distanceTo(ei));
            String line = stack.getName().getString() + " x" + stack.getCount() + "  " + dist + "m";
            int w = tr.getWidth(line);
            ctx.fill(-2, y - 1, w + 4, y + ROW, 0x38000000);
            ctx.drawText(tr, Text.literal(line), 0, y, i == 0 ? 0xFFE8E8E8 : 0xFFAAAAAA, true);
            y += ROW;
        }
        matrices.popMatrix();
    }

    public static int hudHeight() {
        if (!ModernConfig.itemTracker2d) {
            return 10;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        int n = Math.max(1, Math.min(HUD_MAX, nearby(client, RANGE_2D).size()));
        return Math.max(10, Math.round(n * ROW * HUD_SCALE) + 2);
    }

    public static int hudWidth() {
        return 110;
    }

    private static void renderWorld(WorldRenderContext context) {
        if (!ModernConfig.itemTracker3d || context.matrices() == null || context.consumers() == null) {
            return;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.world == null || client.player == null) {
            return;
        }
        CameraRenderState camera = context.worldState().cameraRenderState;
        if (camera == null || !camera.initialized) {
            return;
        }
        float tickDelta = client.getRenderTickCounter().getTickProgress(false);
        Vec3d cam = camera.pos;
        Vec3d look = client.player.getRotationVec(tickDelta);
        Vec3d eye = client.player.getCameraPosVec(tickDelta);
        MatrixStack matrices = context.matrices();
        TextRenderer tr = client.textRenderer;
        int light = LightmapTextureManager.pack(15, 15);
        int shown = 0;
        for (ItemEntity ei : nearby(client, RANGE_3D)) {
            if (shown >= WORLD_MAX) {
                break;
            }
            if (!inView(eye, look, new Vec3d(ei.getX(), ei.getY(), ei.getZ()), 0.55)) {
                continue;
            }
            ItemStack stack = ei.getStack();
            if (stack.isEmpty()) {
                continue;
            }
            double x = ei.lastX + (ei.getX() - ei.lastX) * tickDelta - cam.x;
            double y = ei.lastY + (ei.getY() - ei.lastY) * tickDelta - cam.y + 0.42;
            double z = ei.lastZ + (ei.getZ() - ei.lastZ) * tickDelta - cam.z;
            String label = stack.getCount() > 1
                ? stack.getName().getString() + " x" + stack.getCount()
                : stack.getName().getString();
            double dist = client.player.distanceTo(ei);
            float alpha = (float) Math.max(0.35, 1.0 - dist / RANGE_3D);
            int color = ((int) (alpha * 230.0F) << 24) | 0x00DDDDDD;
            Text text = Text.literal(label);
            matrices.push();
            matrices.translate(x, y, z);
            matrices.multiply(camera.orientation);
            matrices.scale(LABEL_SCALE, -LABEL_SCALE, LABEL_SCALE);
            Matrix4f matrix = matrices.peek().getPositionMatrix();
            float w = tr.getWidth(text) / 2.0F;
            tr.draw(text, -w, 0.0F, color, false, matrix, context.consumers(), TextRenderer.TextLayerType.NORMAL, 0x40000000, light);
            matrices.pop();
            shown++;
        }
    }

    private static boolean inView(Vec3d eye, Vec3d look, Vec3d target, double minDot) {
        Vec3d to = target.subtract(eye);
        double len = to.length();
        if (len < 0.001) {
            return true;
        }
        return to.normalize().dotProduct(look) >= minDot;
    }

    private static List<ItemEntity> nearby(MinecraftClient client, double range) {
        List<ItemEntity> out = new ArrayList<>();
        if (client.world == null || client.player == null) {
            return out;
        }
        for (var e : client.world.getEntities()) {
            if (e instanceof ItemEntity item && client.player.distanceTo(item) <= range) {
                out.add(item);
            }
        }
        out.sort(Comparator.comparingDouble(a -> client.player.squaredDistanceTo(a)));
        return out;
    }
}
