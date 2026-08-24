package com.paraguacraft.pvp.modules;

import com.paraguacraft.pvp.hud.HudDraw;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.renderer.entity.RenderManager;
import net.minecraft.entity.Entity;
import net.minecraft.entity.item.EntityItem;
import net.minecraft.item.ItemStack;
import net.minecraftforge.client.event.RenderWorldLastEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;

import java.util.ArrayList;
import java.util.Collections;
import java.util.Comparator;
import java.util.List;

/** Rastrea items dropeados: lista 2D y etiquetas 3D. */
public final class ItemTracker {

    private static final double RANGE = 24.0D;
    private static final int HUD_MAX = 8;

    public ItemTracker() {}

    public static void drawHud() {
        if (!ModConfig.itemTracker2d) {
            return;
        }
        List<EntityItem> items = nearby();
        int y = 0;
        int n = Math.min(HUD_MAX, items.size());
        for (int i = 0; i < n; i++) {
            EntityItem ei = items.get(i);
            ItemStack stack = ei.getEntityItem();
            if (stack == null) {
                continue;
            }
            String name = stack.getDisplayName();
            int count = stack.stackSize;
            int dist = (int) Minecraft.getMinecraft().thePlayer.getDistanceToEntity(ei);
            HudDraw.labeled(name + " x" + count + " ", dist + "m", 0, y);
            y += 10;
        }
    }

    public static int hudHeight() {
        if (!ModConfig.itemTracker2d) {
            return 10;
        }
        return Math.max(10, Math.min(HUD_MAX, nearby().size()) * 10);
    }

    @SubscribeEvent
    public void onWorldLast(RenderWorldLastEvent event) {
        if (!ModConfig.itemTracker3d) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.theWorld == null || mc.getRenderViewEntity() == null) {
            return;
        }
        float partial = event.partialTicks;
        RenderManager rm = mc.getRenderManager();
        FontRenderer font = rm.getFontRenderer();
        if (font == null) {
            return;
        }
        Entity view = mc.getRenderViewEntity();
        double camX = view.lastTickPosX + (view.posX - view.lastTickPosX) * partial;
        double camY = view.lastTickPosY + (view.posY - view.lastTickPosY) * partial;
        double camZ = view.lastTickPosZ + (view.posZ - view.lastTickPosZ) * partial;
        for (EntityItem ei : nearby()) {
            ItemStack stack = ei.getEntityItem();
            if (stack == null) {
                continue;
            }
            double x = ei.lastTickPosX + (ei.posX - ei.lastTickPosX) * partial - camX;
            double y = ei.lastTickPosY + (ei.posY - ei.lastTickPosY) * partial - camY + 0.45D;
            double z = ei.lastTickPosZ + (ei.posZ - ei.lastTickPosZ) * partial - camZ;
            String label = stack.stackSize > 1
                ? stack.getDisplayName() + " x" + stack.stackSize
                : stack.getDisplayName();
            WaypointManager.drawBillboard(x, y, z, label, rm, font);
        }
    }

    private static List<EntityItem> nearby() {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.theWorld == null || mc.thePlayer == null) {
            return Collections.emptyList();
        }
        List<EntityItem> out = new ArrayList<EntityItem>();
        for (Entity e : mc.theWorld.loadedEntityList) {
            if (e instanceof EntityItem && mc.thePlayer.getDistanceToEntity(e) <= RANGE) {
                out.add((EntityItem) e);
            }
        }
        Collections.sort(out, new Comparator<EntityItem>() {
            @Override
            public int compare(EntityItem a, EntityItem b) {
                return Double.compare(
                    mc.thePlayer.getDistanceSqToEntity(a),
                    mc.thePlayer.getDistanceSqToEntity(b)
                );
            }
        });
        return out;
    }
}
