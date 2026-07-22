package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.Frustum;
import net.minecraft.client.render.entity.state.EntityRenderState;
import net.minecraft.entity.Entity;
import net.minecraft.entity.decoration.ArmorStandEntity;
import net.minecraft.entity.decoration.ItemFrameEntity;
import net.minecraft.entity.player.PlayerEntity;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.Box;

/** Distancias y reglas compartidas de culling (paridad 1.8.9, afinadas para 1.21). */
public final class CullHelper {

    /** Mobs/objetos lejanos fuera de frustum — hubs Hypixel/Cubecraft. */
    public static final int ENTITY_CULL_BLOCKS = 48;
    public static final int ENTITY_CULL_DISTANCE_SQ = ENTITY_CULL_BLOCKS * ENTITY_CULL_BLOCKS;

    /** Nametags de jugadores lejanos. */
    public static final int NAMETAG_CULL_BLOCKS = 32;
    public static final int NAMETAG_CULL_DISTANCE_SQ = NAMETAG_CULL_BLOCKS * NAMETAG_CULL_BLOCKS;

    /** LOD: solo nametag completo si miras al jugador (estilo Lunar). */
    public static final int NAMETAG_LOD_BLOCKS = 18;
    public static final int NAMETAG_LOD_DISTANCE_SQ = NAMETAG_LOD_BLOCKS * NAMETAG_LOD_BLOCKS;

    /** Animaciones congeladas en entidades lejanas. */
    public static final int ENTITY_ANIM_CULL_BLOCKS = 28;
    public static final int ENTITY_ANIM_CULL_DISTANCE_SQ = ENTITY_ANIM_CULL_BLOCKS * ENTITY_ANIM_CULL_BLOCKS;

    /** Cofres, carteles, marcos de mapa, etc. */
    public static final int BLOCK_ENTITY_CULL_BLOCKS = 32;
    public static final int BLOCK_ENTITY_CULL_DISTANCE_SQ = BLOCK_ENTITY_CULL_BLOCKS * BLOCK_ENTITY_CULL_BLOCKS;

    public static final int ARMOR_STAND_CULL_BLOCKS = 40;
    public static final int ARMOR_STAND_CULL_DISTANCE_SQ = ARMOR_STAND_CULL_BLOCKS * ARMOR_STAND_CULL_BLOCKS;

    public static final int ITEM_FRAME_CULL_BLOCKS = 32;
    public static final int ITEM_FRAME_CULL_DISTANCE_SQ = ITEM_FRAME_CULL_BLOCKS * ITEM_FRAME_CULL_BLOCKS;

    private CullHelper() {}

    public static boolean anyCullEnabled() {
        return ModernConfig.entityCull
            || ModernConfig.nametagCull
            || ModernConfig.blockEntityCull
            || ModernConfig.entityAnimCull
            || ModernConfig.armorStandCull
            || ModernConfig.itemFrameCull;
    }

    public static boolean shouldCullEntity(Entity entity, Frustum frustum, double camX, double camY, double camZ) {
        if (entity == null || frustum == null) {
            return false;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.player == null) {
            return false;
        }
        if (entity instanceof PlayerEntity) {
            return false;
        }
        if (entity == client.player || entity.hasPassenger(client.player)) {
            return false;
        }
        if (entity instanceof ArmorStandEntity armorStand) {
            if (ModernConfig.armorStandCull) {
                return squaredDistance(entity, camX, camY, camZ) > ARMOR_STAND_CULL_DISTANCE_SQ;
            }
            return false;
        }
        if (entity instanceof ItemFrameEntity) {
            if (ModernConfig.itemFrameCull) {
                return squaredDistance(entity, camX, camY, camZ) > ITEM_FRAME_CULL_DISTANCE_SQ;
            }
            return false;
        }
        if (!ModernConfig.entityCull) {
            return false;
        }
        double distSq = squaredDistance(entity, camX, camY, camZ);
        if (distSq <= ENTITY_CULL_DISTANCE_SQ) {
            return false;
        }
        Box box = entity.getBoundingBox();
        return !frustum.isVisible(box);
    }

    public static boolean shouldCullNametag(EntityRenderState state, Entity targeted) {
        if (!ModernConfig.nametagCull || state == null) {
            return false;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.player != null && isSamePosition(state, client.player)) {
            return false;
        }
        double distSq = state.squaredDistanceToCamera;
        if (distSq > NAMETAG_CULL_DISTANCE_SQ) {
            return true;
        }
        if (ModernConfig.nametagLod && distSq > NAMETAG_LOD_DISTANCE_SQ) {
            return !isTargetedPlayer(state, targeted);
        }
        return false;
    }

    private static boolean isSamePosition(EntityRenderState state, Entity entity) {
        double dx = entity.getX() - state.x;
        double dy = entity.getY() - state.y;
        double dz = entity.getZ() - state.z;
        return dx * dx + dy * dy + dz * dz < 0.01D;
    }

    private static boolean isTargetedPlayer(EntityRenderState state, Entity targeted) {
        if (!(targeted instanceof PlayerEntity)) {
            return false;
        }
        double dx = targeted.getX() - state.x;
        double dy = targeted.getY() - state.y;
        double dz = targeted.getZ() - state.z;
        return dx * dx + dy * dy + dz * dz < 2.25D;
    }

    public static boolean shouldCullBlockEntity(BlockPos pos, MinecraftClient client) {
        if (!ModernConfig.blockEntityCull || pos == null || client == null || client.player == null) {
            return false;
        }
        double dx = pos.getX() + 0.5D - client.player.getX();
        double dy = pos.getY() + 0.5D - client.player.getEyeY();
        double dz = pos.getZ() + 0.5D - client.player.getZ();
        return dx * dx + dy * dy + dz * dz > BLOCK_ENTITY_CULL_DISTANCE_SQ;
    }

    public static boolean shouldFreezeEntityAnim(Entity entity) {
        if (!ModernConfig.entityAnimCull || entity == null) {
            return false;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.player == null || entity == client.player) {
            return false;
        }
        return client.player.squaredDistanceTo(entity) > ENTITY_ANIM_CULL_DISTANCE_SQ;
    }

    private static double squaredDistance(Entity entity, double camX, double camY, double camZ) {
        double dx = entity.getX() - camX;
        double dy = entity.getY() - camY;
        double dz = entity.getZ() - camZ;
        return dx * dx + dy * dy + dz * dz;
    }
}
