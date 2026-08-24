package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.cosmetics.PlayerTagRenderer;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.inventory.GuiContainer;
import net.minecraft.client.gui.inventory.GuiInventory;
import net.minecraft.entity.EntityLivingBase;
import net.minecraft.inventory.Container;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(GuiInventory.class)
public abstract class MixinGuiInventoryOverlay extends GuiContainer {

    public MixinGuiInventoryOverlay(Container inventorySlotsIn) {
        super(inventorySlotsIn);
    }

    @Inject(method = "drawEntityOnScreen", at = @At("HEAD"), require = 1)
    private static void paraguacraft$beginGuiEntity(
        int posX, int posY, int scale, float mouseX, float mouseY, EntityLivingBase ent, CallbackInfo ci
    ) {
        PlayerTagRenderer.beginGuiEntityPass();
    }

    @Inject(method = "drawEntityOnScreen", at = @At("RETURN"), require = 1)
    private static void paraguacraft$endGuiEntity(
        int posX, int posY, int scale, float mouseX, float mouseY, EntityLivingBase ent, CallbackInfo ci
    ) {
        PlayerTagRenderer.endGuiEntityPass();
    }

    @Inject(method = "drawGuiContainerBackgroundLayer", at = @At("TAIL"))
    private void paraguacraft$playerTag(float partialTicks, int mouseX, int mouseY, CallbackInfo ci) {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null) {
            return;
        }
        // Centro del modelo: vanilla drawEntityOnScreen(guiLeft+51, guiTop+75, 30, ...)
        PlayerTagRenderer.drawInventoryOverlay(this.guiLeft, this.guiTop, mouseX, mouseY, mc.thePlayer);
    }
}
