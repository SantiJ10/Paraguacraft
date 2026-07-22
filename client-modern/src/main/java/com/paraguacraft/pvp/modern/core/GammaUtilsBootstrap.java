package com.paraguacraft.pvp.modern.core;

import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientLifecycleEvents;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.client.MinecraftClient;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/** Activa fullbright de Gamma Utils al arrancar (el valor runtime no se persiste en gammautils.json). */
public final class GammaUtilsBootstrap {

    private static final Logger LOGGER = LoggerFactory.getLogger("ParaguacraftPvP-GammaUtils");
    private static final String MOD_ID = "gammautils";

    private GammaUtilsBootstrap() {}

    public static boolean isLoaded() {
        return FabricLoader.getInstance().isModLoaded(MOD_ID);
    }

    public static void register() {
        if (!isLoaded()) {
            return;
        }
        ClientLifecycleEvents.CLIENT_STARTED.register(client ->
            client.execute(GammaUtilsBootstrap::enableFullbright)
        );
    }

    /** Alterna fullbright via API de Gamma Utils (Mod Menu / tecla G). */
    public static boolean isEnabled() {
        if (!isLoaded()) {
            return false;
        }
        try {
            Class<?> gammaUtils = Class.forName("io.github.sjouwer.gammautils.GammaUtils");
            Object config = gammaUtils.getMethod("getConfig").invoke(null);
            Object gamma = config.getClass().getField("gamma").get(config);
            return (boolean) gamma.getClass().getMethod("isEnabled").invoke(gamma);
        } catch (Throwable ignored) {
            return false;
        }
    }

    public static void toggleFullbright(MinecraftClient client) {
        if (!isLoaded() || client == null) {
            return;
        }
        try {
            Class<?> gammaUtils = Class.forName("io.github.sjouwer.gammautils.GammaUtils");
            Object config = gammaUtils.getMethod("getConfig").invoke(null);
            Object gamma = config.getClass().getField("gamma").get(config);
            boolean enabled = (boolean) gamma.getClass().getMethod("isEnabled").invoke(gamma);
            if (enabled) {
                Class<?> gammaManager = Class.forName("io.github.sjouwer.gammautils.GammaManager");
                gammaManager.getMethod("resetGamma", boolean.class).invoke(null, false);
            } else {
                enableFullbright();
            }
        } catch (Throwable t) {
            LOGGER.debug("No se pudo alternar Gamma Utils", t);
        }
    }

    private static void enableFullbright() {
        try {
            Class<?> gammaUtils = Class.forName("io.github.sjouwer.gammautils.GammaUtils");
            Class<?> gammaManager = Class.forName("io.github.sjouwer.gammautils.GammaManager");
            Object config = gammaUtils.getMethod("getConfig").invoke(null);
            Object gamma = config.getClass().getField("gamma").get(config);
            double toggled = ((Number) gamma.getClass().getMethod("getToggledValue").invoke(gamma)).doubleValue();
            gammaManager
                .getMethod("setGamma", double.class, boolean.class, boolean.class)
                .invoke(null, toggled, false, false);
        } catch (Throwable t) {
            LOGGER.debug("No se pudo activar Gamma Utils al inicio", t);
        }
    }
}
