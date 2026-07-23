package com.paraguacraft.pvp.modern.mixin;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.render.entity.TntEntityRenderer;
import net.minecraft.client.render.entity.state.TntEntityRenderState;
import net.minecraft.entity.TntEntity;
import net.minecraft.text.Text;
import net.minecraft.text.TextColor;
import net.minecraft.util.math.Vec3d;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Segundos restantes flotando sobre la TNT (paridad 1.8.9). */
@Mixin(TntEntityRenderer.class)
public class MixinTntEntityRenderer {

    @Inject(
        method = "updateRenderState(Lnet/minecraft/entity/TntEntity;Lnet/minecraft/client/render/entity/state/TntEntityRenderState;F)V",
        at = @At("RETURN")
    )
    private void paraguacraft$tntCountdownLabel(
        TntEntity entity,
        TntEntityRenderState state,
        float tickDelta,
        CallbackInfo ci
    ) {
        if (!ModernConfig.showTntCountdown || state.fuse <= 0.0F) {
            return;
        }
        int sec = (int) Math.ceil(state.fuse / 20.0F);
        if (sec <= 0) {
            return;
        }
        state.displayName = Text.literal(String.valueOf(sec))
            .styled(s -> s.withColor(TextColor.fromRgb(0xFF5555)));
        if (state.nameLabelPos == null) {
            state.nameLabelPos = new Vec3d(state.x, state.y + state.height + 0.45, state.z);
        }
    }
}
