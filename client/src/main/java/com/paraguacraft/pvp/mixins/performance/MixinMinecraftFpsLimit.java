package com.paraguacraft.pvp.mixins.performance;

import com.paraguacraft.pvp.core.PerformanceConfig;
import net.minecraft.client.Minecraft;
import org.lwjgl.opengl.Display;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Limitador de FPS en segundo plano (estilo Lunar/Badlion).
 *
 * Cuando la ventana está minimizada (o sin foco, si se configura) bajamos el
 * tope de FPS. Esto reduce uso de CPU/GPU y, sobre todo en laptops, evita el
 * thermal throttling que después tira los FPS dentro de la partida.
 *
 * Importante: por defecto SOLO actúa cuando la ventana está minimizada
 * ({@code !Display.isVisible()}), así no molesta al usar el modo borderless con
 * el juego visible en otro monitor.
 */
@Mixin(Minecraft.class)
public class MixinMinecraftFpsLimit {

    @Inject(method = "getLimitFramerate", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$idleFpsLimit(CallbackInfoReturnable<Integer> cir) {
        try {
            if (PerformanceConfig.reduceFpsWhenMinimized && !Display.isVisible()) {
                cir.setReturnValue(Math.max(1, PerformanceConfig.minimizedFps));
                return;
            }
            if (PerformanceConfig.reduceFpsWhenUnfocused && !Display.isActive()) {
                cir.setReturnValue(Math.max(1, PerformanceConfig.unfocusedFps));
            }
        } catch (Throwable ignored) {
        }
    }
}
