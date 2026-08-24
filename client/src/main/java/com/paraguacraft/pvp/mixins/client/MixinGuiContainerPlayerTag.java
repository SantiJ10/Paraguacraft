package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.cosmetics.PlayerTagRenderer;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiScreen;
import net.minecraft.client.gui.inventory.GuiChest;
import net.minecraft.client.gui.inventory.GuiContainer;
import net.minecraft.client.gui.inventory.GuiFurnace;
import net.minecraft.client.gui.inventory.GuiInventory;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(GuiContainer.class)
public abstract class MixinGuiContainerPlayerTag extends GuiScreen {

    @Shadow protected int guiLeft;
    @Shadow protected int guiTop;
    @Shadow protected int xSize;

    @Inject(method = "drawScreen", at = @At("TAIL"))
    private void paraguacraft$playerPreview(int mouseX, int mouseY, float partialTicks, CallbackInfo ci) {
        GuiContainer self = (GuiContainer) (Object) this;
        if (self instanceof GuiInventory) {
            return;
        }
        if (!(self instanceof GuiChest) && !(self instanceof GuiFurnace)) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null) {
            return;
        }
        int feetY = this.guiTop + 80;
        int feetX = this.guiLeft >= 70 ? this.guiLeft - 40 : this.guiLeft + this.xSize + 40;
        PlayerTagRenderer.drawPreview(feetX, feetY, mouseX, mouseY, mc.thePlayer);
    }
}
