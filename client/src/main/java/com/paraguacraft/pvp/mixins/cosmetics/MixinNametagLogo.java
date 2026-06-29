package com.paraguacraft.pvp.mixins.cosmetics;

import com.paraguacraft.pvp.cosmetics.NametagLogoRenderer;
import com.paraguacraft.pvp.modules.ModConfig;
import com.paraguacraft.pvp.network.BadgeRegistry;
import com.paraguacraft.pvp.network.BadgeProtocol;
import net.minecraft.client.Minecraft;
import net.minecraft.client.network.NetworkPlayerInfo;
import net.minecraft.client.renderer.entity.Render;
import net.minecraft.entity.Entity;
import net.minecraft.entity.player.EntityPlayer;
import net.minecraft.util.EnumChatFormatting;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Insignia Paraguacraft en nametags — local + jugadores sincronizados vía plugin.
 * {@code renderLivingLabel} vive en {@link Render} (1.8.9), no en RendererLivingEntity.
 */
@Mixin(Render.class)
public class MixinNametagLogo {

    @ModifyVariable(
        method = "renderLivingLabel(Lnet/minecraft/entity/Entity;Ljava/lang/String;DDDI)V",
        at = @At("HEAD"),
        argsOnly = true,
        ordinal = 1,
        require = 0
    )
    private String paraguacraft$appendPing(String label, Entity entity) {
        if (!ModConfig.showOpponentPing || !(entity instanceof EntityPlayer)) {
            return label;
        }
        EntityPlayer player = (EntityPlayer) entity;
        Minecraft mc = Minecraft.getMinecraft();
        if (player == mc.thePlayer || mc.getNetHandler() == null) {
            return label;
        }
        NetworkPlayerInfo info = mc.getNetHandler().getPlayerInfo(player.getUniqueID());
        if (info == null || info.getResponseTime() < 0) {
            return label;
        }
        return label + EnumChatFormatting.GRAY + " " + info.getResponseTime() + "ms";
    }

    @Inject(
        method = "renderLivingLabel(Lnet/minecraft/entity/Entity;Ljava/lang/String;DDDI)V",
        at = @At(
            value = "INVOKE",
            target = "Lnet/minecraft/client/gui/FontRenderer;drawString(Ljava/lang/String;III)I",
            shift = At.Shift.AFTER
        ),
        require = 0
    )
    private void paraguacraft$drawNametagLogo(
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
            if (!ModConfig.showNametagLogo) {
                return;
            }
            NametagLogoRenderer.drawLeftOfName(mc.getRenderManager().getFontRenderer(), label);
            return;
        }

        if (!ModConfig.showNametagLogoOthers) {
            return;
        }
        if (!BadgeRegistry.hasBadge(player.getUniqueID())) {
            return;
        }
        byte badge = BadgeRegistry.getBadge(player.getUniqueID());
        if (badge == BadgeProtocol.BADGE_NONE) {
            return;
        }
        NametagLogoRenderer.drawLeftOfName(
            mc.getRenderManager().getFontRenderer(),
            label,
            badge
        );
    }
}
