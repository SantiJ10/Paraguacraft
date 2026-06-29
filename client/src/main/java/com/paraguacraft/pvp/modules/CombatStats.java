package com.paraguacraft.pvp.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.entity.Entity;
import net.minecraftforge.event.entity.living.LivingHurtEvent;
import net.minecraftforge.event.entity.player.AttackEntityEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import net.minecraftforge.fml.relauncher.Side;
import net.minecraftforge.fml.relauncher.SideOnly;

/** Reach display y combo counter para entrenamiento PvP. */
@SideOnly(Side.CLIENT)
public class CombatStats {

    public static double lastReach = 0.0;
    public static int comboCount = 0;
    private static long lastHitTime = 0L;

    private static Minecraft mc() {
        return Minecraft.getMinecraft();
    }

    @SubscribeEvent
    public void onAttack(AttackEntityEvent event) {
        if (!ModConfig.reachDisplay && !ModConfig.comboCounter) {
            return;
        }
        Minecraft mc = mc();
        if (mc.thePlayer == null || event.entityPlayer != mc.thePlayer || event.target == null) {
            return;
        }
        Entity target = event.target;
        double dx = target.posX - mc.thePlayer.posX;
        double dy = target.posY + target.getEyeHeight() - (mc.thePlayer.posY + mc.thePlayer.getEyeHeight());
        double dz = target.posZ - mc.thePlayer.posZ;
        lastReach = Math.sqrt(dx * dx + dy * dy + dz * dz);

        if (ModConfig.comboCounter) {
            long now = System.currentTimeMillis();
            if (now - lastHitTime > 3000L) {
                comboCount = 0;
            }
            comboCount++;
            lastHitTime = now;
        }
    }

    @SubscribeEvent
    public void onHurt(LivingHurtEvent event) {
        if (!ModConfig.comboCounter) {
            return;
        }
        Minecraft mc = mc();
        if (mc.thePlayer != null && event.entityLiving == mc.thePlayer) {
            comboCount = 0;
        }
    }

    public static void reset() {
        lastReach = 0.0;
        comboCount = 0;
        lastHitTime = 0L;
    }
}
