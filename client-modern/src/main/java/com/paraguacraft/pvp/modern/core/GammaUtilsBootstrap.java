package com.paraguacraft.pvp.modern.core;

import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientLifecycleEvents;
import net.fabricmc.loader.api.FabricLoader;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/** Activa fullbright de Gamma Utils al arrancar (el valor runtime no se persiste en gammautils.json). */
public final class GammaUtilsBootstrap {

    private static final Logger LOGGER = LoggerFactory.getLogger("ParaguacraftPvP-GammaUtils");
    private static final String MOD_ID = "gammautils";

    private GammaUtilsBootstrap() {}

    public static void register() {
        if (!FabricLoader.getInstance().isModLoaded(MOD_ID)) {
            return;
        }
        ClientLifecycleEvents.CLIENT_STARTED.register(client ->
            client.execute(GammaUtilsBootstrap::enableFullbright)
        );
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
