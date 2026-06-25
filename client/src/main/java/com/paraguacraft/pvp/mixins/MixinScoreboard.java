package com.paraguacraft.pvp.mixins;

import net.minecraft.scoreboard.ScoreObjective;
import net.minecraft.scoreboard.ScorePlayerTeam;
import net.minecraft.scoreboard.Scoreboard;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Hypixel envía removes de teams/objectives que el cliente no tiene registrados;
 * vanilla lanza NPE al recibir null desde getTeam/getObjective.
 */
@Mixin(Scoreboard.class)
public class MixinScoreboard {

    @Inject(method = "removeObjective(Lnet/minecraft/scoreboard/ScoreObjective;)V", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$skipNullObjective(ScoreObjective objective, CallbackInfo ci) {
        if (objective == null) {
            ci.cancel();
        }
    }

    @Inject(method = "removeTeam(Lnet/minecraft/scoreboard/ScorePlayerTeam;)V", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$skipNullTeam(ScorePlayerTeam team, CallbackInfo ci) {
        if (team == null) {
            ci.cancel();
        }
    }
}
