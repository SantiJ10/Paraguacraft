package com.paraguacraft.pvp.core;

import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.OpenGlHelper;
import net.minecraft.client.renderer.texture.TextureManager;
import net.minecraft.util.ResourceLocation;
import org.lwjgl.opengl.GL11;

/** Restauración de estado OpenGL tras overlays 2D (nametags, HUD). */
public final class GlRenderUtil {

    private static final ResourceLocation FONT_ASCII =
        new ResourceLocation("textures/font/ascii.png");

    private GlRenderUtil() {}

    /**
     * Tras dibujar el logo/ping junto al nametag, re-enlaza la textura ASCII
     * sin tocar blend/depth (vanilla sigue limpiando el resto en renderLivingLabel).
     */
    public static void rebindNametagFont() {
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
        GlStateManager.enableTexture2D();
        TextureManager textures = Minecraft.getMinecraft().getTextureManager();
        if (textures != null) {
            textures.bindTexture(FONT_ASCII);
        }
    }

    /**
     * Estado canónico al terminar {@code RenderPlayer#doRender} para que el
     * siguiente jugador no herede textura/color sucios (Steve + nametags rotos en lobby).
     */
    public static void resetAfterPlayerRender() {
        GlStateManager.disableRescaleNormal();
        GlStateManager.setActiveTexture(OpenGlHelper.lightmapTexUnit);
        GlStateManager.enableTexture2D();
        GlStateManager.disableLighting();
        GlStateManager.disableBlend();
        GlStateManager.enableAlpha();
        GlStateManager.tryBlendFuncSeparate(GL11.GL_SRC_ALPHA, GL11.GL_ONE_MINUS_SRC_ALPHA, 1, 0);
        GlStateManager.depthMask(true);
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
        GlStateManager.setActiveTexture(OpenGlHelper.defaultTexUnit);
    }
}
