package com.paraguacraft.pvp.modern.cosmetics;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.gui.CustomPauseScreen;
import com.paraguacraft.pvp.modern.network.ParaguacraftNetwork;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.font.TextRenderer;
import net.minecraft.client.gl.RenderPipelines;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.ingame.AbstractFurnaceScreen;
import net.minecraft.client.gui.screen.ingame.CraftingScreen;
import net.minecraft.client.gui.screen.ingame.GenericContainerScreen;
import net.minecraft.client.gui.screen.ingame.HandledScreen;
import net.minecraft.client.gui.screen.ingame.InventoryScreen;
import net.minecraft.entity.player.PlayerEntity;
import net.minecraft.text.Text;
import org.joml.Matrix3x2fStack;

/**
 * Overlay 2D sobre el modelo 3D (inventario / cofre / pausa) y watermark Lunar.
 * El follow del mouse usa el mismo look vector que {@link InventoryScreen#drawEntity}.
 */
public final class NametagOverlay {

    public static final int HEART = 9;
    private static final int MODEL_SCALE = 30;
    private static final int MODEL_FEET_Y = 78;
    private static final int MODEL_BOX_X1 = 26;
    private static final int MODEL_BOX_Y1 = 8;
    private static final int MODEL_BOX_X2 = 75;
    private static final float OVERLAY_SCALE = 0.7F;
    private static final float LOOK_FOLLOW = 16.0F;

    /** >0 while the GUI 3D player preview is actually rendering. */
    private static int guiEntityDepth;

    private NametagOverlay() {}

    public static void beginGuiEntityPass() {
        guiEntityDepth++;
    }

    public static void endGuiEntityPass() {
        if (guiEntityDepth > 0) {
            guiEntityDepth--;
        }
    }

    public static boolean isGuiEntityPass() {
        return guiEntityDepth > 0;
    }

    public static boolean shouldReplace3dLabel() {
        return ModernConfig.showNametagHealth
            || ModernConfig.showNametagLogo
            || ModernConfig.showNametagLogoOthers;
    }

    /** Evita el nametag 3D del jugador local encima del preview de inventario/pausa. */
    public static boolean isLocalPreviewScreen(Screen screen) {
        return screen instanceof HandledScreen || screen instanceof CustomPauseScreen;
    }

    public static boolean isWatermarkScreen(Screen screen) {
        return screen instanceof InventoryScreen
            || screen instanceof GenericContainerScreen
            || screen instanceof AbstractFurnaceScreen
            || screen instanceof CraftingScreen
            || screen instanceof CustomPauseScreen;
    }

    public static void drawInventoryTag(
        DrawContext context, int guiLeft, int guiTop, int mouseX, int mouseY, PlayerEntity player
    ) {
        drawOnModel(
            context,
            guiLeft + MODEL_BOX_X1,
            guiTop + MODEL_BOX_Y1,
            guiLeft + MODEL_BOX_X2,
            guiTop + MODEL_FEET_Y,
            mouseX,
            mouseY,
            player,
            MODEL_SCALE,
            true,
            true
        );
    }

    /** Preview 3D + overlay para cofres y hornos (pantallas que no dibujan al jugador). */
    public static void drawPreview(
        DrawContext context, int feetX, int feetY, int mouseX, int mouseY, PlayerEntity player
    ) {
        drawPreview(context, feetX, feetY, mouseX, mouseY, player, MODEL_SCALE, true, true, true);
    }

    /** Menú de pausa: solo la skin (sin nombre, logo ni vida). */
    public static void drawPausePreview(
        DrawContext context, int feetX, int feetY, int mouseX, int mouseY, PlayerEntity player
    ) {
        drawPreview(context, feetX, feetY, mouseX, mouseY, player, 42, false, false, false);
    }

    private static void drawPreview(
        DrawContext context, int feetX, int feetY, int mouseX, int mouseY, PlayerEntity player,
        int modelScale, boolean showHealth, boolean showLogo, boolean showName
    ) {
        if (player == null || !ModernConfig.showInventoryTags) {
            return;
        }
        int halfW = Math.round(24.5F * modelScale / (float) MODEL_SCALE);
        int boxH = Math.round(70.0F * modelScale / (float) MODEL_SCALE);
        int x1 = feetX - halfW;
        int y1 = feetY - boxH;
        int x2 = feetX + halfW;
        InventoryScreen.drawEntity(
            context, x1, y1, x2, feetY, modelScale, 0.0625F, (float) mouseX, (float) mouseY, player
        );
        if (showName || showHealth || showLogo) {
            drawOnModel(context, x1, y1, x2, feetY, mouseX, mouseY, player, modelScale, showHealth, showLogo);
        }
    }

    private static void drawOnModel(
        DrawContext context,
        int x1, int y1, int x2, int y2,
        int mouseX, int mouseY,
        PlayerEntity player,
        int modelScale, boolean showHealth, boolean showLogo
    ) {
        if (player == null || !ModernConfig.showInventoryTags) {
            return;
        }
        MinecraftClient client = MinecraftClient.getInstance();
        TextRenderer tr = client.textRenderer;
        float modelCenterX = (x1 + x2) / 2.0F;
        float modelCenterY = (y1 + y2) / 2.0F;
        float follow = LOOK_FOLLOW * (modelScale / (float) MODEL_SCALE);
        float shiftX = -(float) Math.atan((modelCenterX - mouseX) / 40.0F) * follow;
        float shiftY = -(float) Math.atan((modelCenterY - mouseY) / 40.0F) * follow;
        int modelHeadY = y2 - modelScale * 2;
        int nameY = (int) (modelHeadY - tr.fontHeight - 2 + shiftY);
        String name = player.getName().getString();
        int nameW = tr.getWidth(name);
        boolean logo = showLogo && ModernConfig.showNametagLogo && ParaguacraftNetwork.hasLogo(player);

        Matrix3x2fStack matrices = context.getMatrices();
        matrices.pushMatrix();
        matrices.translate(modelCenterX + shiftX, (float) nameY);
        matrices.scale(OVERLAY_SCALE, OVERLAY_SCALE);

        int localNameX = -nameW / 2;
        if (logo) {
            context.drawTexture(
                RenderPipelines.GUI_TEXTURED,
                ParaguacraftTextures.MINI_ICON,
                localNameX - ParaguacraftTextures.MINI_SIZE - 2,
                -1,
                0f,
                0f,
                ParaguacraftTextures.MINI_SIZE,
                ParaguacraftTextures.MINI_SIZE,
                ParaguacraftTextures.MINI_SIZE,
                ParaguacraftTextures.MINI_SIZE
            );
        }
        context.drawText(tr, Text.literal(name), localNameX, 0, 0xFFFFFFFF, true);
        if (showHealth) {
            drawHealth(context, tr, 0, tr.fontHeight + 2, player.getHealth());
        }
        matrices.popMatrix();
    }

    public static void drawWatermark(DrawContext context, int screenW, int screenH) {
        if (!ModernConfig.showWatermark) {
            return;
        }
        int h = ParaguacraftTextures.WATERMARK_H;
        int w = Math.round(h * ParaguacraftTextures.WATERMARK_ASPECT);
        int pad = 8;
        int x = screenW - pad - w;
        int y = screenH - pad - h;
        context.drawTexture(
            RenderPipelines.GUI_TEXTURED,
            ParaguacraftTextures.WATERMARK,
            x,
            y,
            0f,
            0f,
            w,
            h,
            w,
            h
        );
    }

    public static void drawHealth(DrawContext context, TextRenderer tr, int centerX, int y, float health) {
        String hp = String.format("%.1f", Math.max(0.0F, health));
        int w = tr.getWidth(hp);
        int total = HEART + 2 + w;
        int x = centerX - total / 2;
        context.drawGuiTexture(RenderPipelines.GUI_TEXTURED, ParaguacraftTextures.HEART, x, y, HEART, HEART);
        context.drawText(tr, Text.literal(hp), x + HEART + 2, y, 0xFFFF5555, true);
    }
}
