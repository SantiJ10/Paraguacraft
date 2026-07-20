package com.paraguacraft.pvp.modern.mixin;

import net.minecraft.entity.Entity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(Entity.class)
public interface EntityRotationAccessor {
    @Accessor("lastYaw")
    float paraguacraft$getLastYaw();

    @Accessor("lastYaw")
    void paraguacraft$setLastYaw(float yaw);

    @Accessor("lastPitch")
    float paraguacraft$getLastPitch();

    @Accessor("lastPitch")
    void paraguacraft$setLastPitch(float pitch);
}
