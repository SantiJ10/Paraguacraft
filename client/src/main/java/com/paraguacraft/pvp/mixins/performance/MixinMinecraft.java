package com.paraguacraft.pvp.mixins.performance;

import com.paraguacraft.pvp.core.PerformanceConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.WorldClient;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Limpieza de memoria al cambiar de mundo — mitiga los memory leaks clásicos de
 * 1.8.9 (chunks/renderers/WorldClient) que se acumulan al saltar entre lobbies y
 * partidas. Se ejecuta durante la pantalla de carga, no por tick, así que el
 * costo del GC no se nota en juego.
 */
@Mixin(Minecraft.class)
public abstract class MixinMinecraft {

    @Shadow public net.minecraft.client.renderer.RenderGlobal renderGlobal;

    @Inject(
        method = "loadWorld(Lnet/minecraft/client/multiplayer/WorldClient;Ljava/lang/String;)V",
        at = @At("HEAD")
    )
    private void paraguacraft$beforeWorldChange(WorldClient worldClientIn, String loadingMessage, CallbackInfo ci) {
        if (!PerformanceConfig.memoryCleanupOnWorldChange) {
            return;
        }
        Minecraft mc = (Minecraft) (Object) this;
        if (mc.theWorld != null && renderGlobal != null) {
            try {
                renderGlobal.deleteAllDisplayLists();
            } catch (Exception ignored) {
            }
        }
    }

    @Inject(
        method = "loadWorld(Lnet/minecraft/client/multiplayer/WorldClient;Ljava/lang/String;)V",
        at = @At("TAIL")
    )
    private void paraguacraft$afterWorldChange(WorldClient worldClientIn, String loadingMessage, CallbackInfo ci) {
        // Solo al DESCARGAR un mundo (world == null): evita el freeze al ENTRAR a
        // la partida y libera la memoria del lobby/partida anterior.
        if (!PerformanceConfig.memoryCleanupOnWorldChange || worldClientIn != null) {
            return;
        }
        System.gc();
    }
}
