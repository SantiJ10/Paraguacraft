package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.ParaguacraftPvPModern;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gl.RenderPipelines;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.hud.InGameHud;
import net.minecraft.client.render.RenderTickCounter;
import net.minecraft.util.Identifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(InGameHud.class)
public class MixinInGameHudCrosshair {

    @Unique
    private static final Identifier CROSSHAIR_PG = Identifier.of(
        ParaguacraftPvPModern.MOD_ID,
        "textures/gui/crosshair_paraguacraft.png"
    );
    @Unique
    private static final Identifier CROSSHAIR_DEWIER = Identifier.of(
        ParaguacraftPvPModern.MOD_ID,
        "textures/gui/crosshair_dewier.png"
    );

    @Inject(method = "renderCrosshair", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$customCrosshair(DrawContext context, RenderTickCounter tickCounter, CallbackInfo ci) {
        // 0 = Vanilla (resource pack / MC default)
        if (ModernConfig.crosshairMode <= 0) {
            return;
        }
        ci.cancel();
        Identifier tex = ModernConfig.crosshairMode == 2 ? CROSSHAIR_DEWIER : CROSSHAIR_PG;
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.getResourceManager().getResource(tex).isEmpty()) {
            return;
        }
        int size = 16;
        int x = (context.getScaledWindowWidth() - size) / 2;
        int y = (context.getScaledWindowHeight() - size) / 2;
        context.drawTexture(RenderPipelines.GUI_TEXTURED, tex, x, y, 0f, 0f, size, size, size, size);
    }
}
