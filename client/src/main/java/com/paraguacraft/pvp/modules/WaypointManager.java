package com.paraguacraft.pvp.modules;

import com.paraguacraft.pvp.hud.HudDraw;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.Tessellator;
import net.minecraft.client.renderer.WorldRenderer;
import net.minecraft.client.renderer.entity.RenderManager;
import net.minecraft.client.renderer.vertex.DefaultVertexFormats;
import net.minecraft.entity.Entity;
import net.minecraftforge.client.event.RenderWorldLastEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import org.lwjgl.opengl.GL11;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Locale;

/** Waypoints persistentes + HUD 2D + billboard 3D. */
public final class WaypointManager {

    public static final class Waypoint {
        public final String name;
        public final int dim;
        public final double x;
        public final double y;
        public final double z;

        public Waypoint(String name, int dim, double x, double y, double z) {
            this.name = name;
            this.dim = dim;
            this.x = x;
            this.y = y;
            this.z = z;
        }
    }

    private static final List<Waypoint> POINTS = new ArrayList<Waypoint>();
    private static boolean loaded;

    public WaypointManager() {}

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
        Iterator<Waypoint> it = POINTS.iterator();
        boolean found = false;
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

    public static void drawHud() {
        if (!ModConfig.showWaypoints) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null || mc.theWorld == null) {
            return;
        }
        ensureLoaded();
        int dim = mc.thePlayer.dimension;
        int y = 0;
        int shown = 0;
        for (Waypoint wp : POINTS) {
            if (wp.dim != dim) {
                continue;
            }
            double dx = wp.x - mc.thePlayer.posX;
            double dy = wp.y - mc.thePlayer.posY;
            double dz = wp.z - mc.thePlayer.posZ;
            int dist = (int) Math.sqrt(dx * dx + dy * dy + dz * dz);
            HudDraw.labeled(wp.name + " ", dist + "m", 0, y);
            y += 10;
            if (++shown >= 8) {
                break;
            }
        }
    }

    public static int hudHeight() {
        if (!ModConfig.showWaypoints) {
            return 10;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null) {
            return 10;
        }
        int n = 0;
        int dim = mc.thePlayer.dimension;
        for (Waypoint wp : POINTS) {
            if (wp.dim == dim) {
                n++;
            }
        }
        return Math.max(10, Math.min(8, n) * 10);
    }

    @SubscribeEvent
    public void onWorldLast(RenderWorldLastEvent event) {
        if (!ModConfig.showWaypoints) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.theWorld == null || mc.getRenderViewEntity() == null) {
            return;
        }
        ensureLoaded();
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
        int dim = view.dimension;
        for (Waypoint wp : POINTS) {
            if (wp.dim != dim) {
                continue;
            }
            double x = wp.x - camX;
            double y = wp.y - camY + 1.8D;
            double z = wp.z - camZ;
            double dist = Math.sqrt(x * x + y * y + z * z);
            if (dist > 256.0D) {
                continue;
            }
            String label = wp.name + " " + (int) dist + "m";
            drawBillboard(x, y, z, label, rm, font);
        }
    }

    static void drawBillboard(double x, double y, double z, String text, RenderManager rm, FontRenderer font) {
        float scale = 0.026666668F;
        GlStateManager.pushMatrix();
        GlStateManager.translate((float) x, (float) y, (float) z);
        GL11.glNormal3f(0.0F, 1.0F, 0.0F);
        GlStateManager.rotate(-rm.playerViewY, 0.0F, 1.0F, 0.0F);
        GlStateManager.rotate(rm.playerViewX * (Minecraft.getMinecraft().gameSettings.thirdPersonView == 2 ? -1.0F : 1.0F), 1.0F, 0.0F, 0.0F);
        GlStateManager.scale(-scale, -scale, scale);
        GlStateManager.disableLighting();
        GlStateManager.depthMask(false);
        GlStateManager.disableDepth();
        GlStateManager.enableBlend();
        GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
        GlStateManager.disableTexture2D();
        int w = font.getStringWidth(text) / 2;
        Tessellator tess = Tessellator.getInstance();
        WorldRenderer wr = tess.getWorldRenderer();
        wr.begin(7, DefaultVertexFormats.POSITION_COLOR);
        wr.pos(-w - 1, -1.0D, 0.0D).color(0.0F, 0.0F, 0.0F, 0.25F).endVertex();
        wr.pos(-w - 1, 8.0D, 0.0D).color(0.0F, 0.0F, 0.0F, 0.25F).endVertex();
        wr.pos(w + 1, 8.0D, 0.0D).color(0.0F, 0.0F, 0.0F, 0.25F).endVertex();
        wr.pos(w + 1, -1.0D, 0.0D).color(0.0F, 0.0F, 0.0F, 0.25F).endVertex();
        tess.draw();
        GlStateManager.enableTexture2D();
        font.drawString(text, -w, 0, 0x20FFFFFF);
        GlStateManager.enableDepth();
        GlStateManager.depthMask(true);
        font.drawStringWithShadow(text, -w, 0, 0xFF00E5FF);
        GlStateManager.disableBlend();
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
        GlStateManager.enableLighting();
        GlStateManager.popMatrix();
    }

    private static void ensureLoaded() {
        if (loaded) {
            return;
        }
        loaded = true;
        POINTS.clear();
        File file = file();
        if (!file.isFile()) {
            return;
        }
        BufferedReader br = null;
        try {
            br = new BufferedReader(new InputStreamReader(new FileInputStream(file), StandardCharsets.UTF_8));
            String line;
            while ((line = br.readLine()) != null) {
                String[] p = line.split("\\|", -1);
                if (p.length < 5) {
                    continue;
                }
                POINTS.add(new Waypoint(
                    p[0],
                    Integer.parseInt(p[1]),
                    Double.parseDouble(p[2]),
                    Double.parseDouble(p[3]),
                    Double.parseDouble(p[4])
                ));
            }
        } catch (Exception ignored) {
        } finally {
            if (br != null) {
                try {
                    br.close();
                } catch (Exception ignored) {
                }
            }
        }
    }

    private static void save() {
        File file = file();
        file.getParentFile().mkdirs();
        OutputStreamWriter w = null;
        try {
            w = new OutputStreamWriter(new FileOutputStream(file), StandardCharsets.UTF_8);
            for (Waypoint wp : POINTS) {
                w.write(wp.name + "|" + wp.dim + "|"
                    + String.format(Locale.US, "%.2f|%.2f|%.2f", wp.x, wp.y, wp.z)
                    + "\n");
            }
        } catch (Exception ignored) {
        } finally {
            if (w != null) {
                try {
                    w.close();
                } catch (Exception ignored) {
                }
            }
        }
    }

    private static File file() {
        return new File(Minecraft.getMinecraft().mcDataDir, "config/paraguacraft-waypoints.txt");
    }
}
