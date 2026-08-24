package com.paraguacraft.pvp.cosmetics;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.ScaledResolution;
import net.minecraft.client.gui.inventory.GuiContainer;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.Tessellator;
import net.minecraft.client.renderer.WorldRenderer;
import net.minecraft.client.renderer.vertex.DefaultVertexFormats;
import net.minecraft.util.ResourceLocation;
import net.minecraftforge.client.event.GuiScreenEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;

/** Watermark global en contenedores, anclado con {@link ScaledResolution}. */
public final class WatermarkRenderer {

    public static final ResourceLocation ICON =
        new ResourceLocation("paraguacraft", "textures/gui/watermark_icon.png");
    public static final ResourceLocation BANNER =
        new ResourceLocation("paraguacraft", "textures/gui/watermark.png");

    private static final int ICON_SIZE = 8;
    private static final int PAD = 4;

    @SubscribeEvent
    public void onDrawScreen(GuiScreenEvent.DrawScreenEvent.Post event) {
        if (!ModConfig.showWatermark || event.gui == null) {
            return;
        }
        if (!(event.gui instanceof GuiContainer)) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        FontRenderer fr = mc.fontRendererObj;
        if (fr == null) {
            return;
        }
        ScaledResolution sr = new ScaledResolution(mc);
        String text = "Paraguacraft";
        int tw = fr.getStringWidth(text);
        int x = sr.getScaledWidth() - PAD - ICON_SIZE - 3 - tw;
        int y = sr.getScaledHeight() - PAD - ICON_SIZE;

        GlStateManager.pushMatrix();
        try {
            GlStateManager.enableBlend();
            GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
            GlStateManager.color(1.0F, 1.0F, 1.0F, 0.85F);
            GlStateManager.disableLighting();
            GlStateManager.enableTexture2D();
            mc.getTextureManager().bindTexture(ICON);
            Tessellator tess = Tessellator.getInstance();
            WorldRenderer wr = tess.getWorldRenderer();
            wr.begin(7, DefaultVertexFormats.POSITION_TEX);
            wr.pos(x, y + ICON_SIZE, 0.0D).tex(0.0D, 1.0D).endVertex();
            wr.pos(x + ICON_SIZE, y + ICON_SIZE, 0.0D).tex(1.0D, 1.0D).endVertex();
            wr.pos(x + ICON_SIZE, y, 0.0D).tex(1.0D, 0.0D).endVertex();
            wr.pos(x, y, 0.0D).tex(0.0D, 0.0D).endVertex();
            tess.draw();
            GlStateManager.color(1.0F, 1.0F, 1.0F, 0.85F);
            fr.drawStringWithShadow(text, x + ICON_SIZE + 3, y, 0xAAFFFFFF);
        } finally {
            GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
            GlStateManager.popMatrix();
        }
    }
}
