package com.paraguacraft.pvp.modern.network;

import net.minecraft.network.PacketByteBuf;
import net.minecraft.network.codec.PacketCodec;
import net.minecraft.network.packet.CustomPayload;
import net.minecraft.util.Identifier;

/**
 * Payload crudo del canal {@code paraguacraft:bdg}, compatible byte a byte con
 * {@code BadgeProtocol} del cliente 1.8.9 y del plugin Paper (ParaguacraftBadges).
 */
public record BadgePayload(byte[] data) implements CustomPayload {

    public static final CustomPayload.Id<BadgePayload> ID =
        new CustomPayload.Id<>(Identifier.of(BadgeProtocol.CHANNEL_NAMESPACE, BadgeProtocol.CHANNEL_PATH));

    public static final PacketCodec<PacketByteBuf, BadgePayload> CODEC = PacketCodec.of(
        (value, buf) -> buf.writeBytes(value.data()),
        buf -> {
            byte[] bytes = new byte[buf.readableBytes()];
            buf.readBytes(bytes);
            return new BadgePayload(bytes);
        }
    );

    @Override
    public Id<? extends CustomPayload> getId() {
        return ID;
    }
}
