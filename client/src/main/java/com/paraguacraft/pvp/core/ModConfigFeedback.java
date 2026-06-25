package com.paraguacraft.pvp.core;

import net.minecraft.client.Minecraft;
import net.minecraft.util.ChatComponentText;

public final class ModConfigFeedback {

    private ModConfigFeedback() {}

    public static void notifyToggle(String module, boolean enabled) {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null) {
            return;
        }
        String state = enabled ? "\u00A7aON" : "\u00A7cOFF";
        mc.thePlayer.addChatMessage(new ChatComponentText(
            "\u00A78[\u00A79Paraguacraft\u00A78] \u00A77" + module + ": " + state
        ));
    }
}
