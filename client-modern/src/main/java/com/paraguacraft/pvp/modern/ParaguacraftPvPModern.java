package com.paraguacraft.pvp.modern;

import com.paraguacraft.pvp.modern.config.LauncherProfile;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.DiscordPresenceHandler;
import com.paraguacraft.pvp.modern.core.LauncherIpcHandler;
import com.paraguacraft.pvp.modern.core.PerformanceBootstrap;
import com.paraguacraft.pvp.modern.core.QuickPlayState;
import com.paraguacraft.pvp.modern.core.QoLBootstrap;
import com.paraguacraft.pvp.modern.core.TrainingWorldHelper;
import com.paraguacraft.pvp.modern.hud.HudCpsTracker;
import com.paraguacraft.pvp.modern.hud.HudRenderer;
import com.paraguacraft.pvp.modern.input.ModKeybinds;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientLifecycleEvents;
import net.minecraft.client.MinecraftClient;

public class ParaguacraftPvPModern implements ClientModInitializer {

    public static final String MOD_ID = "paraguacraftpvp-modern";
    public static final String VERSION = "0.6.7";

    @Override
    public void onInitializeClient() {
        QuickPlayState.load();
        LauncherProfile.apply();
        PerformanceBootstrap.register();
        QoLBootstrap.register();
        TrainingWorldHelper.register();
        LauncherIpcHandler.register();
        DiscordPresenceHandler.register();
        HudCpsTracker.register();
        HudRenderer.register();
        ModKeybinds.register();
        ClientLifecycleEvents.CLIENT_STARTED.register(client -> applyClientOptions(client));
    }

    private static void applyClientOptions(MinecraftClient client) {
        client.options.getSprintToggled().setValue(ModernConfig.toggleSprint);
    }
}
