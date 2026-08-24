package com.paraguacraft.pvp.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.ScaledResolution;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.util.ResourceLocation;
import net.minecraftforge.client.event.RenderGameOverlayEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;

/**
 * Overlay estilo AppleCore: saturación encima de la barra de hambre vainilla.
 */
public final class SaturationHud extends Gui {

    private static final ResourceLocation ICONS = new ResourceLocation("textures/gui/icons.png");
    private final Minecraft mc = Minecraft.getMinecraft();

    @SubscribeEvent
    public void onFood(RenderGameOverlayEvent.Post event) {
        if (event.type != RenderGameOverlayEvent.ElementType.FOOD) {
            return;
        }
        if (!ModConfig.showSaturation || mc.thePlayer == null || mc.playerController == null) {
            return;
        }
        if (!mc.playerController.gameIsSurvivalOrAdventure()) {
            return;
        }
        if (mc.thePlayer.isRidingHorse()) {
            return;
        }
        float sat = mc.thePlayer.getFoodStats().getSaturationLevel();
        if (sat <= 0.0F) {
            return;
        }
        ScaledResolution sr = new ScaledResolution(mc);
        int left = sr.getScaledWidth() / 2 + 91;
        int top = sr.getScaledHeight() - 39;
        int pips = Math.min(10, (int) Math.ceil(sat / 2.0F));

        GlStateManager.pushMatrix();
        GlStateManager.enableBlend();
        GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
        GlStateManager.color(1.0F, 0.85F, 0.2F, 0.9F);
        mc.getTextureManager().bindTexture(ICONS);
        for (int i = 0; i < pips; i++) {
            int x = left - i * 8 - 9;
            float remain = sat - i * 2.0F;
            if (remain >= 2.0F) {
                this.drawTexturedModalRect(x, top, 52, 27, 9, 9);
            } else if (remain > 0.0F) {
                this.drawTexturedModalRect(x, top, 61, 27, 9, 9);
            }
        }
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
        GlStateManager.popMatrix();
    }
}
