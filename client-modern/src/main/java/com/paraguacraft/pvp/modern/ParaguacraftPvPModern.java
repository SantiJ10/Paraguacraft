package com.paraguacraft.pvp.modern;

import com.paraguacraft.pvp.modern.config.LauncherProfile;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.ColoredBedsBootstrap;
import com.paraguacraft.pvp.modern.core.DiscordPresenceHandler;
import com.paraguacraft.pvp.modern.core.GammaUtilsBootstrap;
import com.paraguacraft.pvp.modern.core.LauncherIpcHandler;
import com.paraguacraft.pvp.modern.core.PerformanceBootstrap;
import com.paraguacraft.pvp.modern.core.SessionBootstrap;
import com.paraguacraft.pvp.modern.core.QoLBootstrap;
import com.paraguacraft.pvp.modern.core.SkinManager;
import com.paraguacraft.pvp.modern.core.TrainingWorldHelper;
import com.paraguacraft.pvp.modern.core.TntCountdownBootstrap;
import com.paraguacraft.pvp.modern.hud.HudCpsTracker;
import com.paraguacraft.pvp.modern.hud.HudRenderer;
import com.paraguacraft.pvp.modern.input.ModKeybinds;
import com.paraguacraft.pvp.modern.core.QuickPlayState;
import com.paraguacraft.pvp.modern.network.BadgeNetHandler;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientLifecycleEvents;
import net.minecraft.client.MinecraftClient;

public class ParaguacraftPvPModern implements ClientModInitializer {

    public static final String MOD_ID = "paraguacraftpvp-modern";
    public static final String VERSION = "0.9.12";

    @Override
    public void onInitializeClient() {
        QuickPlayState.load();
        LauncherProfile.apply();
        // Skins unificadas: si el launcher escribio una skin offline/local, la aplica al arrancar.
        if (ModernConfig.customSkinUrl != null && !ModernConfig.customSkinUrl.isBlank()) {
            SkinManager.apply(ModernConfig.customSkinUrl);
        }
        PerformanceBootstrap.register();
        QoLBootstrap.register();
        SessionBootstrap.register();
        GammaUtilsBootstrap.register();
        TrainingWorldHelper.register();
        LauncherIpcHandler.register();
        DiscordPresenceHandler.register();
        HudCpsTracker.register();
        HudRenderer.register();
        ColoredBedsBootstrap.register();
        TntCountdownBootstrap.register();
        ModKeybinds.register();
        BadgeNetHandler.register();
        ClientLifecycleEvents.CLIENT_STARTED.register(client -> {
            applyClientOptions(client);
            com.paraguacraft.pvp.modern.resourcepack.ResourcePackService.restoreSavedPack();
        });
        ClientLifecycleEvents.CLIENT_STOPPING.register(client -> ModernConfig.save());
    }

    private static void applyClientOptions(MinecraftClient client) {
        client.options.getSprintToggled().setValue(false);
    }
}
