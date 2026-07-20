package com.paraguacraft.badges.paper;

import java.io.ByteArrayOutputStream;
import java.io.DataOutputStream;
import java.io.IOException;
import java.util.Map;
import java.util.UUID;

/** Construye los payloads S2C del canal {@code paraguacraft:bdg} (big-endian, igual que Netty/PacketByteBuf). */
final class BadgeMessages {

    private BadgeMessages() {}

    static byte[] sync(Map<UUID, Byte> badges) {
        try {
            ByteArrayOutputStream out = new ByteArrayOutputStream();
            DataOutputStream data = new DataOutputStream(out);
            data.writeByte(BadgeProtocol.S2C_SYNC);
            data.writeShort(badges.size());
            for (Map.Entry<UUID, Byte> entry : badges.entrySet()) {
                writeUuid(data, entry.getKey());
                data.writeByte(entry.getValue());
            }
            return out.toByteArray();
        } catch (IOException e) {
            return new byte[0];
        }
    }

    static byte[] update(UUID id, byte badge) {
        try {
            ByteArrayOutputStream out = new ByteArrayOutputStream();
            DataOutputStream data = new DataOutputStream(out);
            data.writeByte(BadgeProtocol.S2C_UPDATE);
            writeUuid(data, id);
            data.writeByte(badge);
            return out.toByteArray();
        } catch (IOException e) {
            return new byte[0];
        }
    }

    private static void writeUuid(DataOutputStream data, UUID id) throws IOException {
        data.writeLong(id.getMostSignificantBits());
        data.writeLong(id.getLeastSignificantBits());
    }
}
