package com.paraguacraft.pvp.modern.cosmetics;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.network.ParaguacraftNetwork;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.font.TextRenderer;
import net.minecraft.client.gl.RenderPipelines;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.entity.player.PlayerEntity;
import net.minecraft.text.Text;

/** Overlay 2D (inventario / watermark) con DrawContext. */
public final class NametagOverlay {

    private NametagOverlay() {}

    public static boolean shouldReplace3dLabel() {
        return ModernConfig.showNametagHealth
            || ModernConfig.showNametagLogo
            || ModernConfig.showNametagLogoOthers;
    }

    public static void drawInventoryTag(DrawContext context, int guiLeft, int guiTop, PlayerEntity player) {
        if (player == null || !ModernConfig.showInventoryTags) {
            return;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        TextRenderer tr = client.textRenderer;
        // Vanilla 1.21: drawEntity(x+26, y+8, x+75, y+78, 30, ...)
        int modelCenterX = guiLeft + 51;
        int modelTopY = guiTop + 8;
        int nameY = modelTopY - tr.fontHeight - 2;
        String name = player.getName().getString();
        int nameW = tr.getWidth(name);
        int nameX = modelCenterX - nameW / 2;
        boolean logo = ModernConfig.showNametagLogo && ParaguacraftNetwork.hasLogo(player);

        if (logo) {
            context.drawTexture(
                RenderPipelines.GUI_TEXTURED,
                ParaguacraftTextures.MINI_ICON,
                nameX - ParaguacraftTextures.MINI_SIZE - 2,
                nameY - 1,
                0f,
                0f,
                ParaguacraftTextures.MINI_SIZE,
                ParaguacraftTextures.MINI_SIZE,
                ParaguacraftTextures.TEX_SIZE,
                ParaguacraftTextures.TEX_SIZE,
                ParaguacraftTextures.TEX_SIZE,
                ParaguacraftTextures.TEX_SIZE
            );
        }
        context.drawText(tr, Text.literal(name), nameX, nameY, 0xFFFFFFFF, true);
        drawHealth(context, tr, modelCenterX, nameY + tr.fontHeight + 2, player.getHealth());
    }

    public static void drawWatermark(DrawContext context, int screenW, int screenH) {
        if (!ModernConfig.showWatermark) {
            return;
        }
        int logoW = ParaguacraftTextures.WATERMARK_W;
        int logoH = ParaguacraftTextures.WATERMARK_H;
        int pad = 4;
        int x = screenW - pad - logoW;
        int y = screenH - pad - logoH;
        context.drawTexture(
            RenderPipelines.GUI_TEXTURED,
            ParaguacraftTextures.LOGO,
            x,
            y,
            0f,
            0f,
            logoW,
            logoH,
            ParaguacraftTextures.TEX_SIZE,
            ParaguacraftTextures.TEX_SIZE,
            ParaguacraftTextures.TEX_SIZE,
            ParaguacraftTextures.TEX_SIZE
        );
    }

    public static void drawHealth(DrawContext context, TextRenderer tr, int centerX, int y, float health) {
        String hp = String.format("%.1f", Math.max(0.0F, health));
        int w = tr.getWidth(hp);
        int total = 9 + 2 + w;
        int x = centerX - total / 2;
        context.drawGuiTexture(RenderPipelines.GUI_TEXTURED, ParaguacraftTextures.HEART, x, y, 9, 9);
        context.drawText(tr, Text.literal(hp), x + 11, y, 0xFFFF5555, true);
    }
}
