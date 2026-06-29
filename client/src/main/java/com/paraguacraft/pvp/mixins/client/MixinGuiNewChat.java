package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.modules.CompactChatHandler;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.gui.ChatLine;
import net.minecraft.client.gui.GuiNewChat;
import net.minecraft.util.IChatComponent;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.List;

@Mixin(GuiNewChat.class)
public abstract class MixinGuiNewChat {

    @Shadow @Final private List<ChatLine> chatLines;

    @Inject(method = "printChatMessageWithOptionalDeletion", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$compactChat(IChatComponent chatComponent, int chatLineId, CallbackInfo ci) {
        if (!ModConfig.compactChat || chatComponent == null || chatLines.isEmpty()) {
            return;
        }
        ChatLine last = chatLines.get(0);
        IChatLineAccess access = (IChatLineAccess) (Object) last;
        String incoming = CompactChatHandler.plainText(chatComponent);
        String previous = CompactChatHandler.plainText(access.paraguacraft$getChatComponent());
        if (incoming.isEmpty() || !incoming.equals(previous)) {
            return;
        }
        int count = CompactChatHandler.extractCount(access.paraguacraft$getChatComponent().getFormattedText()) + 1;
        IChatComponent compacted = CompactChatHandler.withCount(access.paraguacraft$getChatComponent(), count);
        chatLines.set(0, new ChatLine(access.paraguacraft$getUpdateCounterCreated(), compacted, access.paraguacraft$getChatLineID()));
        ci.cancel();
    }
}
