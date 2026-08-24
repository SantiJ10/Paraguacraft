package com.paraguacraft.pvp.cosmetics;

import com.paraguacraft.pvp.core.PerformanceConfig;
import com.paraguacraft.pvp.modules.ModConfig;
import com.paraguacraft.pvp.network.BadgeProtocol;
import com.paraguacraft.pvp.network.BadgeRegistry;
import com.paraguacraft.pvp.network.ParaguacraftNetwork;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.network.NetworkPlayerInfo;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.Tessellator;
import net.minecraft.client.renderer.WorldRenderer;
import net.minecraft.client.renderer.entity.RenderManager;
import net.minecraft.client.renderer.vertex.DefaultVertexFormats;
import net.minecraft.entity.player.EntityPlayer;
import net.minecraft.util.ResourceLocation;
import net.minecraftforge.client.event.RenderLivingEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import org.lwjgl.opengl.GL11;

/**
 * Nametags 3D: cancela {@link RenderLivingEvent.Specials.Pre} y redibuja
 * nombre + vida + mini-logo con Tessellator / GlStateManager.
 */
public final class NametagSpecialsRenderer {

    private static final ResourceLocation ICONS = new ResourceLocation("textures/gui/icons.png");
    private static final int HEART = 9;
    private static final int LOGO = NametagLogoRenderer.LOGO_SIZE;

    @SubscribeEvent
    public void onNametag(RenderLivingEvent.Specials.Pre event) {
        if (!(event.entity instanceof EntityPlayer)) {
            return;
        }
        EntityPlayer player = (EntityPlayer) event.entity;
        if (PlayerTagRenderer.isGuiEntityPass() || isLocalPreviewScreen(player)) {
            event.setCanceled(true);
            return;
        }
        if (shouldCull(player)) {
            event.setCanceled(true);
            return;
        }
        event.setCanceled(true);
        draw(player, event.x, event.y, event.z);
    }

    @SubscribeEvent
    public void onLocalNametagF5(RenderLivingEvent.Post event) {
        if (!(event.entity instanceof EntityPlayer)) {
            return;
        }
        EntityPlayer player = (EntityPlayer) event.entity;
        Minecraft mc = Minecraft.getMinecraft();
        if (player != mc.thePlayer || mc.gameSettings.thirdPersonView == 0) {
            return;
        }
        if (PlayerTagRenderer.isGuiEntityPass() || isLocalPreviewScreen(player)) {
            return;
        }
        if (shouldCull(player)) {
            return;
        }
        draw(player, event.x, event.y, event.z);
    }

    private static boolean isLocalPreviewScreen(EntityPlayer player) {
        Minecraft mc = Minecraft.getMinecraft();
        if (player != mc.thePlayer || mc.currentScreen == null) {
            return false;
        }
        return mc.currentScreen instanceof net.minecraft.client.gui.inventory.GuiContainer
            || mc.currentScreen instanceof com.paraguacraft.pvp.gui.CustomPauseMenu;
    }

    private static boolean shouldCull(EntityPlayer player) {
        if (!PerformanceConfig.nametagCull) {
            return false;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null || player == mc.thePlayer) {
            return false;
        }
        double distSq = mc.thePlayer.getDistanceSqToEntity(player);
        if (distSq > PerformanceConfig.nametagCullDistanceSq) {
            return true;
        }
        if (PerformanceConfig.nametagLod && distSq > PerformanceConfig.nametagLodDistanceSq) {
            return mc.objectMouseOver == null || mc.objectMouseOver.entityHit != player;
        }
        return false;
    }

    private static void draw(EntityPlayer player, double x, double y, double z) {
        Minecraft mc = Minecraft.getMinecraft();
        RenderManager rm = mc.getRenderManager();
        FontRenderer font = rm.getFontRenderer();
        if (font == null) {
            return;
        }
        ParaguacraftNetwork.tickLocal();

        String name = player.getDisplayName().getFormattedText();
        boolean local = player == mc.thePlayer;
        boolean logo = false;
        byte badge = BadgeProtocol.BADGE_PARAGUACRAFT;
        if (local) {
            logo = ModConfig.showNametagLogo && ParaguacraftNetwork.hasLogo(player);
        } else if (ModConfig.showNametagLogoOthers && BadgeRegistry.hasBadge(player.getUniqueID())) {
            badge = BadgeRegistry.getBadge(player.getUniqueID());
            logo = badge != BadgeProtocol.BADGE_NONE;
        } else if (ModConfig.showNametagLogo && ParaguacraftNetwork.hasLogo(player)) {
            logo = true;
        }
        boolean health = ModConfig.showNametagHealth;
        String ping = null;
        if (!local && ModConfig.showOpponentPing && mc.getNetHandler() != null) {
            NetworkPlayerInfo info = mc.getNetHandler().getPlayerInfo(player.getUniqueID());
            if (info != null && info.getResponseTime() >= 0) {
                ping = info.getResponseTime() + "ms";
            }
        }

        boolean sneaking = player.isSneaking();
        int nameW = font.getStringWidth(name);
        int half = nameW / 2;
        int logoPad = logo ? LOGO + 2 : 0;
        int pingW = ping != null ? font.getStringWidth(ping) + 2 : 0;
        int left = -half - logoPad - 1;
        int right = half + pingW + 1;
        int top = -1;
        int bottom = 8;
        if (health) {
            bottom += HEART + 3;
        }

        GlStateManager.pushMatrix();
        try {
            GlStateManager.translate((float) x, (float) y + player.height + 0.5F, (float) z);
            GL11.glNormal3f(0.0F, 1.0F, 0.0F);
            float pitchSign = Minecraft.getMinecraft().gameSettings.thirdPersonView == 2 ? -1.0F : 1.0F;
            GlStateManager.rotate(-rm.playerViewY, 0.0F, 1.0F, 0.0F);
            GlStateManager.rotate(rm.playerViewX * pitchSign, 1.0F, 0.0F, 0.0F);
            GlStateManager.scale(-0.026666668F, -0.026666668F, 0.026666668F);
            GlStateManager.disableLighting();
            GlStateManager.enableBlend();
            GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
            GlStateManager.alphaFunc(516, 0.1F);

            if (!sneaking) {
                GlStateManager.depthMask(false);
                GlStateManager.disableDepth();
            }

            GlStateManager.disableTexture2D();
            Tessellator tess = Tessellator.getInstance();
            WorldRenderer wr = tess.getWorldRenderer();
            wr.begin(7, DefaultVertexFormats.POSITION_COLOR);
            wr.pos(left, top, 0.0D).color(0.0F, 0.0F, 0.0F, 0.25F).endVertex();
            wr.pos(left, bottom, 0.0D).color(0.0F, 0.0F, 0.0F, 0.25F).endVertex();
            wr.pos(right, bottom, 0.0D).color(0.0F, 0.0F, 0.0F, 0.25F).endVertex();
            wr.pos(right, top, 0.0D).color(0.0F, 0.0F, 0.0F, 0.25F).endVertex();
            tess.draw();
            GlStateManager.enableTexture2D();

            font.drawString(name, -half, 0, 553648127);
            if (ping != null) {
                font.drawString(ping, half + 2, 0, 0x20AAAAAA);
            }

            if (!sneaking) {
                GlStateManager.enableDepth();
                GlStateManager.depthMask(true);
            }
            font.drawString(name, -half, 0, sneaking ? 553648127 : -1);
            if (ping != null) {
                font.drawString(ping, half + 2, 0, sneaking ? 0x20AAAAAA : 0xFFAAAAAA);
            }

            if (logo) {
                drawLogo(-half - LOGO - 2, (font.FONT_HEIGHT - LOGO) / 2 - 1, badge);
            }
            if (health) {
                drawHealth(font, 0, font.FONT_HEIGHT + 1, player.getHealth());
            }
        } finally {
            GlStateManager.enableDepth();
            GlStateManager.depthMask(true);
            GlStateManager.enableTexture2D();
            GlStateManager.disableBlend();
            GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
            GlStateManager.enableLighting();
            GlStateManager.popMatrix();
        }
    }

    private static void drawLogo(int x, int y, byte badge) {
        GlStateManager.enableTexture2D();
        GlStateManager.enableBlend();
        GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
        if (badge == BadgeProtocol.BADGE_STAFF) {
            GlStateManager.color(1.0F, 0.85F, 0.2F, 1.0F);
        } else {
            GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
        }
        Minecraft.getMinecraft().getTextureManager().bindTexture(NametagLogoRenderer.MINI_ICON);
        texturedQuad(x, y, LOGO, LOGO, 0, 0, LOGO, LOGO, LOGO, LOGO);
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
    }

    private static void drawHealth(FontRenderer font, int centerX, int y, float health) {
        String hp = String.format("%.1f", Math.max(0.0F, health));
        int w = font.getStringWidth(hp);
        int total = HEART + 2 + w;
        int x = centerX - total / 2;
        GlStateManager.enableTexture2D();
        GlStateManager.enableBlend();
        GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
        Minecraft.getMinecraft().getTextureManager().bindTexture(ICONS);
        texturedQuad(x, y, HEART, HEART, 16, 0, HEART, HEART, 256, 256);
        texturedQuad(x, y, HEART, HEART, 52, 0, HEART, HEART, 256, 256);
        font.drawString(hp, x + HEART + 2, y, 0xFFFF5555);
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
    }

    private static void texturedQuad(
        float x,
        float y,
        float w,
        float h,
        float u,
        float v,
        float uw,
        float vh,
        float texW,
        float texH
    ) {
        float u0 = u / texW;
        float v0 = v / texH;
        float u1 = (u + uw) / texW;
        float v1 = (v + vh) / texH;
        Tessellator tess = Tessellator.getInstance();
        WorldRenderer wr = tess.getWorldRenderer();
        wr.begin(7, DefaultVertexFormats.POSITION_TEX);
        wr.pos(x, y + h, 0.0D).tex(u0, v1).endVertex();
        wr.pos(x + w, y + h, 0.0D).tex(u1, v1).endVertex();
        wr.pos(x + w, y, 0.0D).tex(u1, v0).endVertex();
        wr.pos(x, y, 0.0D).tex(u0, v0).endVertex();
        tess.draw();
    }
}
