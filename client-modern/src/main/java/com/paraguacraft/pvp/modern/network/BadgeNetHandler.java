package com.paraguacraft.pvp.modern.network;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayNetworking;
import net.fabricmc.fabric.api.networking.v1.PacketByteBufs;
import net.fabricmc.fabric.api.networking.v1.PayloadTypeRegistry;
import net.minecraft.client.MinecraftClient;
import net.minecraft.network.PacketByteBuf;

import java.util.UUID;

/**
 * Port de {@code BadgeNetHandler} 1.8.9: registra el canal {@code paraguacraft:bdg},
 * envia el registro del jugador y sincroniza insignias recibidas en {@link BadgeRegistry}.
 */
public final class BadgeNetHandler {

    private BadgeNetHandler() {}

    public static void register() {
        PayloadTypeRegistry.playC2S().register(BadgePayload.ID, BadgePayload.CODEC);
        PayloadTypeRegistry.playS2C().register(BadgePayload.ID, BadgePayload.CODEC);

        ClientPlayConnectionEvents.JOIN.register((handler, sender, client) -> {
            BadgeRegistry.clear();
            client.execute(BadgeNetHandler::sendRegister);
        });
        ClientPlayConnectionEvents.DISCONNECT.register((handler, client) -> BadgeRegistry.clear());

        ClientPlayNetworking.registerGlobalReceiver(BadgePayload.ID, (payload, context) ->
            context.client().execute(() -> handlePayload(payload)));
    }

    private static void handlePayload(BadgePayload payload) {
        PacketByteBuf buf = new PacketByteBuf(io.netty.buffer.Unpooled.wrappedBuffer(payload.data()));
        if (!buf.isReadable()) {
            return;
        }
        byte type = buf.readByte();
        if (type == BadgeProtocol.S2C_SYNC) {
            if (!buf.isReadable(2)) {
                return;
            }
            int count = buf.readShort();
            for (int i = 0; i < count; i++) {
                if (!buf.isReadable(17)) {
                    break;
                }
                UUID id = readUuid(buf);
                byte badge = buf.readByte();
                BadgeRegistry.set(id, badge);
            }
        } else if (type == BadgeProtocol.S2C_UPDATE) {
            if (!buf.isReadable(17)) {
                return;
            }
            UUID id = readUuid(buf);
            byte badge = buf.readByte();
            BadgeRegistry.set(id, badge);
        }
    }

    public static void sendRegister() {
        if (!ModernConfig.showNametagLogo) {
            return;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.player == null || client.getNetworkHandler() == null) {
            return;
        }
        if (client.isIntegratedServerRunning()) {
            BadgeRegistry.set(client.player.getUuid(), BadgeProtocol.BADGE_PARAGUACRAFT);
            return;
        }
        if (!ClientPlayNetworking.canSend(BadgePayload.ID)) {
            return;
        }
        PacketByteBuf buf = PacketByteBufs.create();
        buf.writeByte(BadgeProtocol.C2S_REGISTER);
        buf.writeByte(BadgeProtocol.BADGE_PARAGUACRAFT);
        byte[] data = new byte[buf.readableBytes()];
        buf.readBytes(data);
        ClientPlayNetworking.send(new BadgePayload(data));
    }

    private static UUID readUuid(PacketByteBuf buf) {
        return new UUID(buf.readLong(), buf.readLong());
    }
}
