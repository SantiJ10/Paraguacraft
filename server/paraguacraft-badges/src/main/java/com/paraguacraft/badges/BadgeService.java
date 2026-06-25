package com.paraguacraft.badges;

import org.bukkit.Bukkit;
import org.bukkit.entity.Player;

import java.io.ByteArrayOutputStream;
import java.io.DataOutputStream;
import java.io.IOException;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

public final class BadgeService {

    private final Map<UUID, Byte> badges = new ConcurrentHashMap<UUID, Byte>();

    public void setBadge(UUID id, byte badge) {
        if (badge == BadgeProtocol.BADGE_NONE) {
            badges.remove(id);
        } else {
            badges.put(id, badge);
        }
    }

    public byte getBadge(UUID id) {
        Byte b = badges.get(id);
        return b != null ? b : BadgeProtocol.BADGE_NONE;
    }

    public void remove(UUID id) {
        badges.remove(id);
    }

    public void sendFullSync(Player target) {
        byte[] payload = buildSync();
        if (payload.length > 0) {
            target.sendPluginMessage(ParaguacraftBadgesPlugin.getInstance(), BadgeProtocol.CHANNEL, payload);
        }
    }

    public void broadcastUpdate(UUID id, byte badge) {
        byte[] payload = buildUpdate(id, badge);
        for (Player online : Bukkit.getOnlinePlayers()) {
            online.sendPluginMessage(ParaguacraftBadgesPlugin.getInstance(), BadgeProtocol.CHANNEL, payload);
        }
    }

    private byte[] buildSync() {
        try {
            ByteArrayOutputStream bytes = new ByteArrayOutputStream();
            DataOutputStream out = new DataOutputStream(bytes);
            out.writeByte(BadgeProtocol.S2C_SYNC);
            out.writeShort(badges.size());
            for (Map.Entry<UUID, Byte> entry : badges.entrySet()) {
                writeUuid(out, entry.getKey());
                out.writeByte(entry.getValue());
            }
            out.flush();
            return bytes.toByteArray();
        } catch (IOException e) {
            return new byte[0];
        }
    }

    private byte[] buildUpdate(UUID id, byte badge) {
        try {
            ByteArrayOutputStream bytes = new ByteArrayOutputStream();
            DataOutputStream out = new DataOutputStream(bytes);
            out.writeByte(BadgeProtocol.S2C_UPDATE);
            writeUuid(out, id);
            out.writeByte(badge);
            out.flush();
            return bytes.toByteArray();
        } catch (IOException e) {
            return new byte[0];
        }
    }

    static void writeUuid(DataOutputStream out, UUID id) throws IOException {
        out.writeLong(id.getMostSignificantBits());
        out.writeLong(id.getLeastSignificantBits());
    }

    static UUID readUuid(java.io.DataInputStream in) throws IOException {
        return new UUID(in.readLong(), in.readLong());
    }
}
