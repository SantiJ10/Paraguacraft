package com.paraguacraft.pvp.mixins.cosmetics;

import com.paraguacraft.pvp.cosmetics.NametagLogoRenderer;
import com.paraguacraft.pvp.modules.ModConfig;
import com.paraguacraft.pvp.network.BadgeRegistry;
import com.paraguacraft.pvp.network.BadgeProtocol;
import net.minecraft.client.Minecraft;
import net.minecraft.client.network.NetworkPlayerInfo;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.entity.Render;
import net.minecraft.client.renderer.texture.TextureMap;
import net.minecraft.entity.Entity;
import net.minecraft.entity.player.EntityPlayer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Insignia Paraguacraft + ping rival en nametags — local y jugadores sincronizados.
 * {@code renderLivingLabel} vive en {@link Render} (1.8.9).
 *
 * El ping se dibuja de forma aditiva a la derecha del nombre (sin modificar el
 * String del nametag) para evitar inyecciones frágiles que rompían el arranque.
 */
@Mixin(Render.class)
public class MixinNametagLogo {

    @Inject(
        method = "renderLivingLabel(Lnet/minecraft/entity/Entity;Ljava/lang/String;DDDI)V",
        at = @At(
            value = "INVOKE",
            target = "Lnet/minecraft/client/gui/FontRenderer;drawString(Ljava/lang/String;III)I",
            shift = At.Shift.AFTER
        ),
        require = 0
    )
    private void paraguacraft$drawNametagExtras(
        Entity entity,
        String label,
        double x,
        double y,
        double z,
        int maxDistance,
        CallbackInfo ci
    ) {
        if (!(entity instanceof EntityPlayer)) {
            return;
        }
        EntityPlayer player = (EntityPlayer) entity;
        Minecraft mc = Minecraft.getMinecraft();
        boolean isLocal = player == mc.thePlayer;

        if (isLocal) {
            if (ModConfig.showNametagLogo) {
                NametagLogoRenderer.drawLeftOfName(mc.getRenderManager().getFontRenderer(), label);
            }
            return;
        }

        if (ModConfig.showNametagLogoOthers && BadgeRegistry.hasBadge(player.getUniqueID())) {
            byte badge = BadgeRegistry.getBadge(player.getUniqueID());
            if (badge != BadgeProtocol.BADGE_NONE) {
                NametagLogoRenderer.drawLeftOfName(
                    mc.getRenderManager().getFontRenderer(),
                    label,
                    badge
                );
            }
        }

        if (ModConfig.showOpponentPing && mc.getNetHandler() != null) {
            NetworkPlayerInfo info = mc.getNetHandler().getPlayerInfo(player.getUniqueID());
            if (info != null && info.getResponseTime() >= 0) {
                NametagLogoRenderer.drawRightOfName(
                    mc.getRenderManager().getFontRenderer(),
                    label,
                    info.getResponseTime() + "ms",
                    0xFFAAAAAA
                );
            }
        }
    }

    @Inject(method = "renderLivingLabel(Lnet/minecraft/entity/Entity;Ljava/lang/String;DDDI)V", at = @At("RETURN"))
    private void paraguacraft$resetAfterNametag(Entity entity, String label, double x, double y, double z, int maxDistance, CallbackInfo ci) {
        if (!(entity instanceof EntityPlayer)) {
            return;
        }
        GlStateManager.color(1.0F, 1.0F, 1.0F, 1.0F);
        GlStateManager.enableTexture2D();
        GlStateManager.disableBlend();
        Minecraft mc = Minecraft.getMinecraft();
        if (mc != null && mc.getTextureManager() != null) {
            mc.getTextureManager().bindTexture(TextureMap.locationBlocksTexture);
        }
    }
}
