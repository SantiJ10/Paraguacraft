package com.paraguacraft.pvp.command;

import com.paraguacraft.pvp.modules.ChatAlerts;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.command.CommandBase;
import net.minecraft.command.ICommandSender;
import net.minecraft.util.BlockPos;
import net.minecraft.util.ChatComponentText;
import net.minecraft.util.EnumChatFormatting;

import java.util.Arrays;
import java.util.List;

/**
 * Comando cliente: /chat alerts ...
 * Hace intuitivo el modulo de alertas (agregar palabras, sonido, color, resaltado).
 */
public class CommandChatAlerts extends CommandBase {

    @Override
    public String getCommandName() {
        return "chat";
    }

    @Override
    public List<String> getCommandAliases() {
        return Arrays.asList("alerts", "alertas");
    }

    @Override
    public String getCommandUsage(ICommandSender sender) {
        return "/chat alerts <add|del|list|on|off|sound|color|clear>";
    }

    @Override
    public int getRequiredPermissionLevel() {
        return 0;
    }

    @Override
    public boolean canCommandSenderUseCommand(ICommandSender sender) {
        return true;
    }

    @Override
    public void processCommand(ICommandSender sender, String[] args) {
        // Soporta /chat alerts ... y los alias /alerts ...
        String[] a = args;
        if (a.length > 0 && (a[0].equalsIgnoreCase("alerts") || a[0].equalsIgnoreCase("alertas"))) {
            a = Arrays.copyOfRange(a, 1, a.length);
        }
        if (a.length == 0) {
            help(sender);
            return;
        }
        String sub = a[0].toLowerCase();

        if (sub.equals("add") || sub.equals("agregar")) {
            if (a.length < 2) {
                msg(sender, EnumChatFormatting.RED + "Uso: " + EnumChatFormatting.GRAY + "/chat alerts add <palabra>");
                return;
            }
            String word = join(a, 1);
            boolean ok = ChatAlerts.add(word);
            ChatAlerts.enabled = true;
            ChatAlerts.save();
            ModConfig.chatTriggers = true;
            if (ok) {
                msg(sender, EnumChatFormatting.GREEN + "Alerta agregada: " + EnumChatFormatting.YELLOW + word);
            } else {
                msg(sender, EnumChatFormatting.GRAY + "Esa palabra ya estaba en la lista.");
            }
        } else if (sub.equals("del") || sub.equals("remove") || sub.equals("rem") || sub.equals("quitar")) {
            if (a.length < 2) {
                msg(sender, EnumChatFormatting.RED + "Uso: " + EnumChatFormatting.GRAY + "/chat alerts del <palabra>");
                return;
            }
            String word = join(a, 1);
            boolean ok = ChatAlerts.remove(word);
            msg(sender, ok
                ? EnumChatFormatting.GREEN + "Alerta eliminada: " + EnumChatFormatting.YELLOW + word
                : EnumChatFormatting.GRAY + "No estaba en la lista: " + word);
        } else if (sub.equals("list") || sub.equals("lista")) {
            List<String> words = ChatAlerts.words();
            if (words.isEmpty()) {
                msg(sender, EnumChatFormatting.GRAY + "No hay alertas. Agrega una con " + EnumChatFormatting.YELLOW + "/chat alerts add <palabra>");
            } else {
                msg(sender, EnumChatFormatting.AQUA + "Alertas (" + words.size() + "): " + EnumChatFormatting.YELLOW + join(words.toArray(new String[0]), 0));
            }
        } else if (sub.equals("on")) {
            ChatAlerts.enabled = true;
            ChatAlerts.save();
            ModConfig.chatTriggers = true;
            msg(sender, EnumChatFormatting.GREEN + "Alertas activadas.");
        } else if (sub.equals("off")) {
            ChatAlerts.enabled = false;
            ChatAlerts.save();
            msg(sender, EnumChatFormatting.RED + "Alertas desactivadas.");
        } else if (sub.equals("sound") || sub.equals("sonido")) {
            ChatAlerts.sound = !ChatAlerts.sound;
            ChatAlerts.save();
            msg(sender, EnumChatFormatting.AQUA + "Sonido de alerta: " + state(ChatAlerts.sound));
        } else if (sub.equals("highlight") || sub.equals("resaltar")) {
            ChatAlerts.highlight = !ChatAlerts.highlight;
            ChatAlerts.save();
            msg(sender, EnumChatFormatting.AQUA + "Resaltado de mensaje: " + state(ChatAlerts.highlight));
        } else if (sub.equals("color")) {
            if (a.length < 2) {
                msg(sender, EnumChatFormatting.RED + "Uso: " + EnumChatFormatting.GRAY + "/chat alerts color <yellow|red|gold|aqua|...>");
                return;
            }
            boolean ok = ChatAlerts.setColor(a[1]);
            msg(sender, ok
                ? EnumChatFormatting.GREEN + "Color de alerta: " + ChatAlerts.colorFmt() + ChatAlerts.color
                : EnumChatFormatting.RED + "Color invalido. Ej: yellow, red, gold, aqua, light_purple.");
        } else if (sub.equals("clear") || sub.equals("limpiar")) {
            ChatAlerts.clear();
            msg(sender, EnumChatFormatting.GREEN + "Lista de alertas vaciada.");
        } else {
            help(sender);
        }
    }

    @Override
    public List<String> addTabCompletionOptions(ICommandSender sender, String[] args, BlockPos pos) {
        if (args.length == 1) {
            return getListOfStringsMatchingLastWord(args, "alerts");
        }
        if (args.length == 2) {
            return getListOfStringsMatchingLastWord(args, "add", "del", "list", "on", "off", "sound", "highlight", "color", "clear");
        }
        if (args.length == 3 && args[1].equalsIgnoreCase("color")) {
            return getListOfStringsMatchingLastWord(args, "yellow", "red", "gold", "aqua", "green", "light_purple", "white");
        }
        return null;
    }

    private static void help(ICommandSender sender) {
        msg(sender, EnumChatFormatting.AQUA + "" + EnumChatFormatting.BOLD + "Paraguacraft Chat Alerts");
        msg(sender, EnumChatFormatting.GRAY + "Te avisa (sonido + resaltado) cuando alguien escribe una palabra clave.");
        msg(sender, EnumChatFormatting.YELLOW + "/chat alerts add <palabra>" + EnumChatFormatting.GRAY + " - agrega una palabra");
        msg(sender, EnumChatFormatting.YELLOW + "/chat alerts del <palabra>" + EnumChatFormatting.GRAY + " - elimina una palabra");
        msg(sender, EnumChatFormatting.YELLOW + "/chat alerts list" + EnumChatFormatting.GRAY + " - lista tus palabras");
        msg(sender, EnumChatFormatting.YELLOW + "/chat alerts sound|highlight|on|off" + EnumChatFormatting.GRAY + " - opciones");
        msg(sender, EnumChatFormatting.YELLOW + "/chat alerts color <color>" + EnumChatFormatting.GRAY + " - color del resaltado");
    }

    private static String state(boolean on) {
        return on ? EnumChatFormatting.GREEN + "ON" : EnumChatFormatting.RED + "OFF";
    }

    private static String join(String[] arr, int from) {
        StringBuilder sb = new StringBuilder();
        for (int i = from; i < arr.length; i++) {
            if (sb.length() > 0) {
                sb.append(' ');
            }
            sb.append(arr[i]);
        }
        return sb.toString();
    }

    private static void msg(ICommandSender sender, String text) {
        sender.addChatMessage(new ChatComponentText(text));
    }
}
