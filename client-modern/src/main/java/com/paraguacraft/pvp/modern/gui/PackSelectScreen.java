package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.ParaguacraftPvPModern;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import com.paraguacraft.pvp.modern.resourcepack.CatalogLoader;
import com.paraguacraft.pvp.modern.resourcepack.CatalogPack;
import com.paraguacraft.pvp.modern.resourcepack.ResourcePackService;
import net.minecraft.client.gl.RenderPipelines;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;
import net.minecraft.util.Identifier;

/** Selector de texture packs PvP 1.21.11. */
public class PackSelectScreen extends ParaguacraftScreen {

    private static final Identifier PACK_ICON = Identifier.of(ParaguacraftPvPModern.MOD_ID, "textures/gui/mod_icon.png");

    private String status = "";
    private CatalogPack[] packs = new CatalogPack[0];

    public PackSelectScreen(Screen parent) {
        super(Text.literal("Texture packs PvP"), parent);
    }

    @Override
    protected void init() {
        packs = CatalogLoader.getFeatured();
        rebuild();
    }

    private void rebuild() {
        clearChildren();
        int btnW = 260;
        int btnH = 22;
        int startY = 72;
        int gap = 24;
        for (int i = 0; i < packs.length; i++) {
            CatalogPack pack = packs[i];
            int y = startY + i * gap;
            boolean installed = ResourcePackService.isInstalled(pack.fileName);
            String label = pack.title + (installed ? " ✓" : " ↓");
            addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, y, btnW, btnH,
                Text.literal(label), () -> onPack(pack, installed)));
        }
        int after = startY + packs.length * gap + 8;
        addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, after, btnW, btnH,
            Text.literal("Volver"), () -> client.setScreen(parent)));
    }

    private void onPack(CatalogPack pack, boolean installed) {
        if (installed) {
            ResourcePackService.applyPack(pack.fileName);
            status = "Activado: " + pack.title + " (Opciones > Resource Packs)";
        } else {
            status = "Descargando " + pack.title + "…";
            ResourcePackService.download(pack, new ResourcePackService.ProgressListener() {
                @Override
                public void onProgress(String s, float ratio) {
                    status = s;
                }

                @Override
                public void onComplete(String fileName) {
                    status = "Listo: " + pack.title;
                    client.execute(PackSelectScreen.this::rebuild);
                }

                @Override
                public void onError(String message) {
                    status = "Error: " + message;
                }
            });
        }
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        super.render(context, mouseX, mouseY, delta);
        context.drawCenteredTextWithShadow(
            textRenderer,
            Text.literal("Texture packs PvP 1.21.11"),
            width / 2,
            48,
            UiTheme.accent()
        );
        int iconSize = 32;
        int listX = width / 2 - 130;
        int startY = 72;
        int gap = 24;
        for (int i = 0; i < packs.length; i++) {
            int y = startY + i * gap;
            if ("paraguacraft-pvp".equals(packs[i].id)) {
                context.drawTexture(RenderPipelines.GUI_TEXTURED, PACK_ICON,
                    listX - iconSize - 8, y - 5, 0f, 0f, iconSize, iconSize, iconSize, iconSize);
            }
        }
        if (!status.isEmpty()) {
            context.drawCenteredTextWithShadow(textRenderer, Text.literal(status), width / 2, height - 56, UiTheme.textDim());
        }
    }
}
