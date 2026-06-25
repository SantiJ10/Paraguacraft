package com.paraguacraft.pvp.mixins.bootstrap;

import net.minecraftforge.fml.relauncher.IFMLLoadingPlugin;

import java.util.Map;

/**
 * CoreMod: garantiza orden de carga temprano. La config Mixin vive en el manifest del JAR
 * ({@code MixinConfigs} + {@code TweakClass}).
 */
public class MixinBootstrap implements IFMLLoadingPlugin {

    @Override
    public String[] getASMTransformerClass() {
        return new String[0];
    }

    @Override
    public String getModContainerClass() {
        return null;
    }

    @Override
    public String getSetupClass() {
        return null;
    }

    @Override
    public void injectData(Map<String, Object> data) {
    }

    @Override
    public String getAccessTransformerClass() {
        return null;
    }
}
