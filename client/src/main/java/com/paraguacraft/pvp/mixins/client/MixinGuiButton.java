package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.gui.theme.UiTheme;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiButton;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Look flat Paraguacraft + hover celeste en botones vanilla (y GuiOptionButton).
 */
@Mixin(GuiButton.class)
public abstract class MixinGuiButton {

    @Shadow public int xPosition;
    @Shadow public int yPosition;
    @Shadow public int width;
    @Shadow public int height;
    @Shadow public String displayString;
    @Shadow public boolean visible;
    @Shadow public boolean enabled;
    @Shadow protected boolean hovered;

    @Inject(method = "drawButton", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$styledDraw(Minecraft mc, int mouseX, int mouseY, CallbackInfo ci) {
        String cn = ((Object) this).getClass().getName();
        if (cn.contains("Slider") || cn.contains("TextField") || cn.contains("Scrollbar")) {
            return;
        }
        if (!this.visible) {
            ci.cancel();
            return;
        }

        this.hovered = mouseX >= this.xPosition
            && mouseY >= this.yPosition
            && mouseX < this.xPosition + this.width
            && mouseY < this.yPosition + this.height;

        boolean hover = this.hovered && this.enabled;
        int bg = !this.enabled ? 0x88444444 : (hover ? UiTheme.BTN_HOVER : UiTheme.BTN_BG);
        Gui.drawRect(this.xPosition, this.yPosition, this.xPosition + this.width, this.yPosition + this.height, bg);

        int border = hover ? UiTheme.ACCENT : 0x55FFFFFF;
        Gui.drawRect(this.xPosition, this.yPosition, this.xPosition + this.width, this.yPosition + 1, border);
        Gui.drawRect(
            this.xPosition,
            this.yPosition + this.height - 1,
            this.xPosition + this.width,
            this.yPosition + this.height,
            border
        );
        Gui.drawRect(this.xPosition, this.yPosition, this.xPosition + 1, this.yPosition + this.height, border);
        Gui.drawRect(
            this.xPosition + this.width - 1,
            this.yPosition,
            this.xPosition + this.width,
            this.yPosition + this.height,
            border
        );

        int textColor = !this.enabled ? UiTheme.TEXT_DIM : (hover ? UiTheme.ACCENT : UiTheme.TEXT);
        int tx = this.xPosition + (this.width - mc.fontRendererObj.getStringWidth(this.displayString)) / 2;
        int ty = this.yPosition + (this.height - 8) / 2;
        mc.fontRendererObj.drawStringWithShadow(this.displayString, tx, ty, textColor);
        ci.cancel();
    }
}
