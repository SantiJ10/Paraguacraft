package com.paraguacraft.pvp.modern.mixin;

import net.minecraft.client.gui.screen.world.CreateWorldScreen;
import net.minecraft.client.gui.screen.world.WorldCreator;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;
import org.spongepowered.asm.mixin.gen.Invoker;

@Mixin(CreateWorldScreen.class)
public interface CreateWorldScreenAccessor {

    @Accessor("worldCreator")
    WorldCreator paraguacraft$getWorldCreator();

    @Invoker("createLevel")
    void paraguacraft$createLevel();
}
