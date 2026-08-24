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
import org.joml.Matrix4f;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

public final class ItemTracker {

    private static final double RANGE = 24.0;
    private static final int HUD_MAX = 8;

    private ItemTracker() {}

    public static void register() {
        WorldRenderEvents.AFTER_ENTITIES.register(ItemTracker::renderWorld);
    }

    public static void drawHud(DrawContext ctx, TextRenderer tr) {
        if (!ModernConfig.itemTracker2d) {
            return;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        List<ItemEntity> items = nearby(client);
        int y = 0;
        int n = Math.min(HUD_MAX, items.size());
        for (int i = 0; i < n; i++) {
            ItemEntity ei = items.get(i);
            ItemStack stack = ei.getStack();
            if (stack.isEmpty()) {
                continue;
            }
            int dist = (int) client.player.distanceTo(ei);
            String line = stack.getName().getString() + " x" + stack.getCount() + " " + dist + "m";
            ctx.drawText(tr, Text.literal(line), 0, y, 0xFFFFFFFF, true);
            y += 10;
        }
    }

    public static int hudHeight() {
        return HUD_MAX * 10;
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
        MatrixStack matrices = context.matrices();
        TextRenderer tr = client.textRenderer;
        int light = LightmapTextureManager.pack(15, 15);
        for (ItemEntity ei : nearby(client)) {
            ItemStack stack = ei.getStack();
            if (stack.isEmpty()) {
                continue;
            }
            double x = ei.lastX + (ei.getX() - ei.lastX) * tickDelta - cam.x;
            double y = ei.lastY + (ei.getY() - ei.lastY) * tickDelta - cam.y + 0.45;
            double z = ei.lastZ + (ei.getZ() - ei.lastZ) * tickDelta - cam.z;
            String label = stack.getCount() > 1
                ? stack.getName().getString() + " x" + stack.getCount()
                : stack.getName().getString();
            Text text = Text.literal(label);
            matrices.push();
            matrices.translate(x, y, z);
            matrices.multiply(camera.orientation);
            matrices.scale(0.025F, -0.025F, 0.025F);
            Matrix4f matrix = matrices.peek().getPositionMatrix();
            float w = tr.getWidth(text) / 2.0F;
            tr.draw(text, -w, 0.0F, 0xFFFFFFFF, false, matrix, context.consumers(), TextRenderer.TextLayerType.NORMAL, 0x40000000, light);
            matrices.pop();
        }
    }

    private static List<ItemEntity> nearby(MinecraftClient client) {
        List<ItemEntity> out = new ArrayList<>();
        if (client.world == null || client.player == null) {
            return out;
        }
        for (var e : client.world.getEntities()) {
            if (e instanceof ItemEntity item && client.player.distanceTo(item) <= RANGE) {
                out.add(item);
            }
        }
        out.sort(Comparator.comparingDouble(a -> client.player.squaredDistanceTo(a)));
        return out;
    }
}
