package com.paraguacraft.pvp.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.item.ItemStack;
import net.minecraft.util.ChatComponentText;
import net.minecraft.util.EnumChatFormatting;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import net.minecraftforge.fml.common.gameevent.TickEvent;

/** Alerta de chat cuando una pieza baja del 25% (con cooldown). */
public final class ArmorDurabilityAlert {

    private static final String[] SLOT = {"Botas", "Pantalones", "Peto", "Casco"};
    private static final long COOLDOWN_MS = 8000L;
    private final long[] lastAlert = new long[4];
    private final Minecraft mc = Minecraft.getMinecraft();

    @SubscribeEvent
    public void onTick(TickEvent.ClientTickEvent event) {
        if (event.phase != TickEvent.Phase.END) {
            return;
        }
        if (!ModConfig.armorDurabilityAlert || mc.thePlayer == null) {
            return;
        }
        long now = System.currentTimeMillis();
        ItemStack[] armor = mc.thePlayer.inventory.armorInventory;
        for (int i = 0; i < 4; i++) {
            ItemStack stack = armor[i];
            if (stack == null || !stack.isItemStackDamageable()) {
                continue;
            }
            int max = stack.getMaxDamage();
            if (max <= 0) {
                continue;
            }
            int percent = (int) (((max - stack.getItemDamage()) * 100.0F) / max);
            if (percent >= 25) {
                continue;
            }
            if (now - lastAlert[i] < COOLDOWN_MS) {
                continue;
            }
            lastAlert[i] = now;
            mc.thePlayer.addChatMessage(new ChatComponentText(
                EnumChatFormatting.RED + "[Armadura] " + EnumChatFormatting.YELLOW + SLOT[i]
                    + EnumChatFormatting.RED + " al " + percent + "%"
            ));
        }
    }
}
