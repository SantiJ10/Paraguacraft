package com.paraguacraft.pvp.network;

import com.paraguacraft.pvp.modules.ModConfig;
import io.netty.buffer.Unpooled;
import net.minecraft.client.Minecraft;
import net.minecraft.network.PacketBuffer;
import net.minecraft.network.play.client.C17PacketCustomPayload;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import net.minecraftforge.fml.common.network.FMLNetworkEvent;

import java.util.UUID;

public class BadgeNetHandler {

    @SubscribeEvent
    public void onConnect(FMLNetworkEvent.ClientConnectedToServerEvent event) {
        BadgeRegistry.clear();
        Minecraft.getMinecraft().addScheduledTask(new Runnable() {
            @Override
            public void run() {
                sendRegister();
            }
        });
    }

    @SubscribeEvent
    public void onDisconnect(FMLNetworkEvent.ClientDisconnectionFromServerEvent event) {
        BadgeRegistry.clear();
    }

    @SubscribeEvent
    public void onCustomPacket(FMLNetworkEvent.ClientCustomPacketEvent event) {
        if (!BadgeProtocol.CHANNEL.equals(event.packet.channel())) {
            return;
        }
        PacketBuffer buf = new PacketBuffer(event.packet.payload());
        if (buf == null || !buf.isReadable()) {
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
        if (!ModConfig.showNametagLogo) {
            return;
        }
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer == null || mc.getNetHandler() == null) {
            return;
        }
        if (mc.isIntegratedServerRunning()) {
            BadgeRegistry.set(mc.thePlayer.getUniqueID(), BadgeProtocol.BADGE_PARAGUACRAFT);
            return;
        }
        PacketBuffer buf = new PacketBuffer(Unpooled.buffer());
        buf.writeByte(BadgeProtocol.C2S_REGISTER);
        buf.writeByte(BadgeProtocol.BADGE_PARAGUACRAFT);
        mc.getNetHandler().addToSendQueue(new C17PacketCustomPayload(BadgeProtocol.CHANNEL, buf));
    }

    private static UUID readUuid(PacketBuffer buf) {
        return new UUID(buf.readLong(), buf.readLong());
    }
}
