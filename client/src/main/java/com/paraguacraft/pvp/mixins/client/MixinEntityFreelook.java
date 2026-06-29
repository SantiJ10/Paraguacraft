package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.modules.FreelookManager;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.entity.Entity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Freelook no baneable.
 *
 * {@code Entity.setAngles(deltaX, deltaY)} es el unico punto donde el mouse rota
 * al jugador. Interceptamos AQUI (en la clase donde realmente se declara el
 * metodo, no en EntityPlayerSP) para que el redirect aplique siempre.
 *
 * Cuando freelook esta activo y el que rota es el jugador local: cancelamos la
 * rotacion del cuerpo y mandamos el delta SOLO a la camara. Resultado:
 *  - El cuerpo del jugador NO rota -> el servidor recibe la rotacion real congelada.
 *  - getMouseOver/raytrace usan la rotacion real del cuerpo (no la camara),
 *    porque la camara solo se sobreescribe dentro de orientCamera al renderizar.
 * Por eso es indetectable: el servidor nunca ve angulos anomalos.
 */
@Mixin(Entity.class)
public class MixinEntityFreelook {

    @Inject(method = "setAngles(FF)V", at = @At("HEAD"), cancellable = true, require = 0)
    private void paraguacraft$freelookAbsorb(float yaw, float pitch, CallbackInfo ci) {
        if (!ModConfig.freelookEnabled || !FreelookManager.active) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc == null || mc.thePlayer == null) {
            return;
        }
        if ((Object) this == mc.thePlayer) {
            FreelookManager.addMouseDelta(yaw, pitch);
            ci.cancel();
        }
    }
}
