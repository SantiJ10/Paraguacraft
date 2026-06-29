package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.network.NetworkPlayerInfo;
import net.minecraft.client.renderer.entity.Render;
import net.minecraft.entity.Entity;
import net.minecraft.entity.player.EntityPlayer;
import net.minecraft.util.EnumChatFormatting;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

@Mixin(Render.class)
public class MixinRenderLivingLabelPing {

    @ModifyVariable(
        method = "renderLivingLabel(Lnet/minecraft/entity/Entity;Ljava/lang/String;DDDI)V",
        at = @At("HEAD"),
        argsOnly = true,
        ordinal = 1
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
        if (info == null) {
            return label;
        }
        int ping = info.getResponseTime();
        if (ping < 0) {
            return label;
        }
        return label + EnumChatFormatting.GRAY + " " + ping + "ms";
    }
}
