package com.paraguacraft.pvp.mixins;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.entity.RenderManager;
import net.minecraft.client.renderer.entity.RenderTNTPrimed;
import net.minecraft.entity.item.EntityTNTPrimed;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(RenderTNTPrimed.class)
public abstract class MixinRenderTNTPrimed {

    @Inject(method = "doRender", at = @At("RETURN"))
    private void paraguacraft$tntCountdown(
        EntityTNTPrimed entity,
        double x,
        double y,
        double z,
        float entityYaw,
        float partialTicks,
        CallbackInfo ci
    ) {
        if (!ModConfig.showTntCountdown || entity.fuse <= 0) {
            return;
        }
        RenderManager rm = Minecraft.getMinecraft().getRenderManager();
        FontRenderer font = rm.getFontRenderer();
        if (font == null) {
            return;
        }
        float sec = entity.fuse / 20.0f;
        int seconds = (int) Math.ceil(sec);
        if (seconds <= 0) {
            return;
        }
        String text = String.valueOf(seconds);

        GlStateManager.pushMatrix();
        GlStateManager.translate((float) x, (float) y + entity.height + 0.5f, (float) z);
        GlStateManager.rotate(-rm.playerViewY, 0.0F, 1.0F, 0.0F);
        GlStateManager.rotate((float) (rm.playerViewX * (Minecraft.getMinecraft().gameSettings.thirdPersonView == 2 ? -1 : 1)), 1.0F, 0.0F, 0.0F);
        GlStateManager.scale(-0.025F, -0.025F, 0.025F);
        GlStateManager.disableLighting();
        GlStateManager.disableDepth();
        GlStateManager.enableBlend();
        GlStateManager.tryBlendFuncSeparate(770, 771, 1, 0);
        font.drawString(text, -font.getStringWidth(text) / 2, 0, 0xFFFF5555);
        GlStateManager.disableBlend();
        GlStateManager.enableDepth();
        GlStateManager.enableLighting();
        GlStateManager.popMatrix();
    }
}
