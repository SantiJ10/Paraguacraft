package com.paraguacraft.pvp.modules;

import com.paraguacraft.pvp.core.PerformanceConfig;
import net.minecraft.block.Block;
import net.minecraft.block.material.Material;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.RenderGlobal;
import net.minecraft.util.AxisAlignedBB;
import net.minecraft.util.BlockPos;
import net.minecraft.util.MovingObjectPosition;
import net.minecraftforge.client.event.DrawBlockHighlightEvent;
import net.minecraftforge.client.event.FOVUpdateEvent;
import net.minecraftforge.client.event.RenderGameOverlayEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import net.minecraftforge.fml.common.gameevent.TickEvent.ClientTickEvent;
import org.lwjgl.opengl.GL11;

public class VisualsManager {
    private final Minecraft mc = Minecraft.getMinecraft();
    private static final net.minecraft.util.ResourceLocation TIGHTFAULT_TEX = new net.minecraft.util.ResourceLocation("paraguacraft", "textures/gui/tightfault.png");

    @SubscribeEvent
    public void onBlockHighlight(DrawBlockHighlightEvent event) {
        if (event.target.typeOfHit == MovingObjectPosition.MovingObjectType.BLOCK) {
            BlockPos pos = event.target.getBlockPos();
            Block block = mc.theWorld.getBlockState(pos).getBlock();
            
            if (block.getMaterial() != Material.air) {
                event.setCanceled(true); 
                
                GlStateManager.enableBlend();
                GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
                GL11.glLineWidth(3.0F); 
                GlStateManager.disableTexture2D();
                GlStateManager.depthMask(false);

                GL11.glColor4f(0.0F, 0.898F, 1.0F, 0.9F);
                
                double d0 = mc.getRenderManager().viewerPosX;
                double d1 = mc.getRenderManager().viewerPosY;
                double d2 = mc.getRenderManager().viewerPosZ;
                AxisAlignedBB bb = block.getSelectedBoundingBox(mc.theWorld, pos).expand(0.002D, 0.002D, 0.002D).offset(-d0, -d1, -d2);
                RenderGlobal.drawSelectionBoundingBox(bb);
                
                GlStateManager.depthMask(true);
                GlStateManager.enableTexture2D();
                GlStateManager.disableBlend();
                GL11.glColor4f(1.0F, 1.0F, 1.0F, 1.0F);
            }
        }
    }

    @SubscribeEvent
    public void onClientTick(ClientTickEvent event) {
        if (mc.theWorld != null) {
            // NOTA: ya NO forzamos setWorldTime(6000) cada tick. Eso peleaba con las
            // actualizaciones de hora del servidor (S03PacketTimeUpdate, ~1/seg) y con la
            // animacion de cielo del fin de partida en Bedwars, provocando el parpadeo /
            // quiebre de texturas. El brillo se maneja por gamma (fullbright), no por hora.
            if (PerformanceConfig.boostFps) {
                mc.theWorld.setRainStrength(0.0F);
                mc.theWorld.setThunderStrength(0.0F);
            }
        }

        // Toggle sneak: vive en QoLManager (LivingUpdate) para respuesta instantánea.
    }

    @SubscribeEvent
    public void onFOVUpdate(FOVUpdateEvent event) {
        if (ModConfig.dynamicFov) {
            event.newfov = 1.0F; 
        }
    }

    @SubscribeEvent
    public void onRenderCrosshair(RenderGameOverlayEvent.Pre event) {
        if (event.type == RenderGameOverlayEvent.ElementType.CROSSHAIRS) {
            if (ModConfig.crosshairMode == 0) return; 

            event.setCanceled(true); 
            
            net.minecraft.client.gui.ScaledResolution sr = new net.minecraft.client.gui.ScaledResolution(mc);
            int cx = sr.getScaledWidth() / 2;
            int cy = sr.getScaledHeight() / 2;
            int color = 0xFF00E5FF; 
            
            if (ModConfig.crosshairMode == 1) { 
                net.minecraft.client.gui.Gui.drawRect(cx - 4, cy - 1, cx + 4, cy + 1, color); 
                net.minecraft.client.gui.Gui.drawRect(cx - 1, cy - 4, cx + 1, cy + 4, color); 
            } 
            else if (ModConfig.crosshairMode == 2) { 
                int gap = 2, size = 5, thickness = 1;
                net.minecraft.client.gui.Gui.drawRect(cx - gap - size, cy - thickness, cx - gap, cy + thickness, color);
                net.minecraft.client.gui.Gui.drawRect(cx + gap, cy - thickness, cx + gap + size, cy + thickness, color);
                net.minecraft.client.gui.Gui.drawRect(cx - thickness, cy - gap - size, cx + thickness, cy - gap, color);
                net.minecraft.client.gui.Gui.drawRect(cx - thickness, cy + gap, cx + thickness, cy + gap + size, color);
            } 
            else if (ModConfig.crosshairMode == 3) { 
                net.minecraft.client.gui.Gui.drawRect(cx - 1, cy - 1, cx + 1, cy + 1, color);
            }
            else if (ModConfig.crosshairMode == 4) { 
                GlStateManager.enableBlend();
                GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
                GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F); 
                mc.getTextureManager().bindTexture(TIGHTFAULT_TEX);
                net.minecraft.client.gui.Gui.drawModalRectWithCustomSizedTexture(cx - 8, cy - 8, 0, 0, 16, 16, 16, 16);
                GlStateManager.disableBlend();
            }
        }
    }
}