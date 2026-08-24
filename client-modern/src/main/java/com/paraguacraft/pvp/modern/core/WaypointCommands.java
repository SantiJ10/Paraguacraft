package com.paraguacraft.pvp.modern.core;

import com.mojang.brigadier.arguments.StringArgumentType;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.fabricmc.fabric.api.client.command.v2.ClientCommandManager;
import net.fabricmc.fabric.api.client.command.v2.ClientCommandRegistrationCallback;
import net.minecraft.client.MinecraftClient;
import net.minecraft.text.Text;
import net.minecraft.util.Formatting;

public final class WaypointCommands {

    private WaypointCommands() {}

    public static void register() {
        ClientCommandRegistrationCallback.EVENT.register((dispatcher, registryAccess) -> {
            var wp = ClientCommandManager.literal("wp")
                .then(ClientCommandManager.literal("add")
                    .executes(ctx -> add(null))
                    .then(ClientCommandManager.argument("name", StringArgumentType.greedyString())
                        .executes(ctx -> add(StringArgumentType.getString(ctx, "name")))))
                .then(ClientCommandManager.literal("del")
                    .then(ClientCommandManager.argument("name", StringArgumentType.greedyString())
                        .executes(ctx -> {
                            String name = StringArgumentType.getString(ctx, "name");
                            boolean ok = WaypointManager.remove(name);
                            msg(ok ? "Eliminado: " + name : "No existe: " + name, ok ? Formatting.GREEN : Formatting.GRAY);
                            return 1;
                        })))
                .then(ClientCommandManager.literal("list")
                    .executes(ctx -> {
                        var all = WaypointManager.all();
                        if (all.isEmpty()) {
                            msg("Sin waypoints. /wp add <nombre>", Formatting.GRAY);
                            return 1;
                        }
                        msg("Waypoints (" + all.size() + "):", Formatting.AQUA);
                        for (var p : all) {
                            msg(p.name() + "  " + (int) p.x() + " " + (int) p.y() + " " + (int) p.z(), Formatting.YELLOW);
                        }
                        return 1;
                    }));
            dispatcher.register(wp);
            dispatcher.register(ClientCommandManager.literal("waypoint").redirect(dispatcher.getRoot().getChild("wp")));
        });
    }

    private static int add(String name) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.player == null || client.world == null) {
            return 0;
        }
        if (name == null || name.isBlank()) {
            name = "wp" + (WaypointManager.all().size() + 1);
        }
        int dim = client.world.getRegistryKey().getValue().toString().hashCode();
        WaypointManager.add(name, dim, client.player.getX(), client.player.getY(), client.player.getZ());
        ModernConfig.showWaypoints = true;
        msg("Waypoint " + name + " guardado.", Formatting.GREEN);
        return 1;
    }

    private static void msg(String text, Formatting color) {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.player != null) {
            client.player.sendMessage(Text.literal(text).formatted(color), false);
        }
    }
}
