package com.paraguacraft.pvp.command;

import com.paraguacraft.pvp.modules.ModConfig;
import com.paraguacraft.pvp.modules.WaypointManager;
import net.minecraft.client.Minecraft;
import net.minecraft.command.CommandBase;
import net.minecraft.command.ICommandSender;
import net.minecraft.entity.player.EntityPlayer;
import net.minecraft.util.BlockPos;
import net.minecraft.util.ChatComponentText;
import net.minecraft.util.EnumChatFormatting;

import java.util.Arrays;
import java.util.List;

public class CommandWaypoint extends CommandBase {

    @Override
    public String getCommandName() {
        return "wp";
    }

    @Override
    public List<String> getCommandAliases() {
        return Arrays.asList("waypoint", "waypoints");
    }

    @Override
    public String getCommandUsage(ICommandSender sender) {
        return "/wp <add|del|list> [nombre]";
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
        if (args.length == 0) {
            help(sender);
            return;
        }
        String sub = args[0].toLowerCase();
        EntityPlayer player = Minecraft.getMinecraft().thePlayer;
        if (sub.equals("add") || sub.equals("agregar")) {
            if (player == null) {
                return;
            }
            String name = args.length >= 2 ? join(args, 1) : "wp" + (WaypointManager.all().size() + 1);
            WaypointManager.add(name, player.dimension, player.posX, player.posY, player.posZ);
            ModConfig.showWaypoints = true;
            msg(sender, EnumChatFormatting.GREEN + "Waypoint " + EnumChatFormatting.YELLOW + name
                + EnumChatFormatting.GREEN + " guardado.");
        } else if (sub.equals("del") || sub.equals("remove") || sub.equals("quitar")) {
            if (args.length < 2) {
                msg(sender, EnumChatFormatting.RED + "Uso: /wp del <nombre>");
                return;
            }
            String name = join(args, 1);
            boolean ok = WaypointManager.remove(name);
            msg(sender, ok
                ? EnumChatFormatting.GREEN + "Eliminado: " + EnumChatFormatting.YELLOW + name
                : EnumChatFormatting.GRAY + "No existe: " + name);
        } else if (sub.equals("list") || sub.equals("lista")) {
            List<WaypointManager.Waypoint> all = WaypointManager.all();
            if (all.isEmpty()) {
                msg(sender, EnumChatFormatting.GRAY + "Sin waypoints. /wp add <nombre>");
                return;
            }
            msg(sender, EnumChatFormatting.AQUA + "Waypoints (" + all.size() + "):");
            for (WaypointManager.Waypoint wp : all) {
                msg(sender, EnumChatFormatting.YELLOW + wp.name + EnumChatFormatting.GRAY
                    + "  " + (int) wp.x + " " + (int) wp.y + " " + (int) wp.z + "  dim " + wp.dim);
            }
        } else {
            help(sender);
        }
    }

    @Override
    public List<String> addTabCompletionOptions(ICommandSender sender, String[] args, BlockPos pos) {
        if (args.length == 1) {
            return getListOfStringsMatchingLastWord(args, "add", "del", "list");
        }
        return null;
    }

    private static void help(ICommandSender sender) {
        msg(sender, EnumChatFormatting.AQUA + "Paraguacraft Waypoints");
        msg(sender, EnumChatFormatting.YELLOW + "/wp add [nombre]" + EnumChatFormatting.GRAY + " — marca tu posición");
        msg(sender, EnumChatFormatting.YELLOW + "/wp del <nombre>" + EnumChatFormatting.GRAY + " — borra");
        msg(sender, EnumChatFormatting.YELLOW + "/wp list" + EnumChatFormatting.GRAY + " — lista");
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
