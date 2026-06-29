package com.paraguacraft.pvp.modules;

import com.paraguacraft.pvp.core.HypixelHelper;
import net.minecraft.client.Minecraft;
import net.minecraft.util.ChatComponentText;
import net.minecraft.util.EnumChatFormatting;
import net.minecraftforge.client.event.ClientChatReceivedEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;

/** Alertas visuales en chat segun reglas configurables + alertas por palabra clave. */
public class ChatTriggerManager {

    private final Minecraft mc = Minecraft.getMinecraft();

    @SubscribeEvent
    public void onChat(ClientChatReceivedEvent event) {
        // type 2 = mensaje sobre la hotbar (action bar): no lo tocamos para no romperlo.
        if (event.message == null || event.type == 2) {
            return;
        }
        String plain = EnumChatFormatting.getTextWithoutFormattingCodes(event.message.getFormattedText()).toLowerCase();
        if (plain.isEmpty()) {
            return;
        }

        // --- Chat Alerts simples (comando /chat alerts) ---
        if (ChatAlerts.enabled) {
            String hit = ChatAlerts.firstMatch(plain);
            if (hit != null) {
                if (ChatAlerts.sound) {
                    playDing();
                }
                if (ChatAlerts.highlight) {
                    EnumChatFormatting c = ChatAlerts.colorFmt();
                    String raw = EnumChatFormatting.getTextWithoutFormattingCodes(event.message.getFormattedText());
                    event.message = new ChatComponentText(
                        c.toString() + EnumChatFormatting.BOLD + "[!] " + EnumChatFormatting.RESET + c + raw);
                }
            }
        }

        if (!ModConfig.chatTriggers) {
            return;
        }
        ChatTriggerConfig.ensureLoaded();

        for (ChatTriggerConfig.Rule rule : ChatTriggerConfig.getRules()) {
            if (!rule.enabled) {
                continue;
            }
            if (rule.hypixelOnly && !HypixelHelper.isOnHypixel()) {
                continue;
            }
            for (String keyword : ChatTriggerConfig.keywords(rule)) {
                if (plain.contains(keyword)) {
                    showTitle(formatTitle(rule), "", rule.fadeIn, rule.stay, rule.fadeOut);
                    return;
                }
            }
        }
    }

    private static String formatTitle(ChatTriggerConfig.Rule rule) {
        EnumChatFormatting fmt = parseColor(rule.color);
        return fmt + "" + EnumChatFormatting.BOLD + rule.title;
    }

    private static EnumChatFormatting parseColor(String name) {
        if (name == null) {
            return EnumChatFormatting.RED;
        }
        try {
            return EnumChatFormatting.valueOf(name.trim().toUpperCase());
        } catch (IllegalArgumentException e) {
            return EnumChatFormatting.RED;
        }
    }

    private void showTitle(String title, String subtitle, int fadeIn, int stay, int fadeOut) {
        if (mc.ingameGUI != null) {
            mc.ingameGUI.displayTitle(title, subtitle, fadeIn, stay, fadeOut);
        }
    }

    private void playDing() {
        if (mc.thePlayer != null) {
            mc.thePlayer.playSound("note.pling", 1.0F, 1.0F);
        }
    }
}
