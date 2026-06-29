package com.paraguacraft.pvp.mixins.client;

import net.minecraft.client.gui.ChatLine;
import net.minecraft.util.IChatComponent;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(ChatLine.class)
public interface IChatLineAccess {

    @Accessor("updateCounterCreated")
    int paraguacraft$getUpdateCounterCreated();

    @Accessor("lineString")
    IChatComponent paraguacraft$getChatComponent();

    @Accessor("chatLineID")
    int paraguacraft$getChatLineID();
}
