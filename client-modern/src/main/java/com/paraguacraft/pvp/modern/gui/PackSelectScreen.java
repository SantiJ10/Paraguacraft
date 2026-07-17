package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import com.paraguacraft.pvp.modern.resourcepack.CatalogLoader;
import com.paraguacraft.pvp.modern.resourcepack.CatalogPack;
import com.paraguacraft.pvp.modern.resourcepack.ResourcePackService;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

/** Selector de texture packs PvP 1.21.11. */
public class PackSelectScreen extends Screen {

    private final Screen parent;
    private String status = "";
    private CatalogPack[] packs = new CatalogPack[0];

    public PackSelectScreen(Screen parent) {
        super(Text.literal("Texture packs PvP"));
        this.parent = parent;
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
            addDrawableChild(ButtonWidget.builder(Text.literal(label), b -> {
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
            }).dimensions(width / 2 - btnW / 2, y, btnW, btnH).build());
        }
        int after = startY + packs.length * gap + 8;
        addDrawableChild(ButtonWidget.builder(Text.literal("Volver"), b ->
            client.setScreen(parent)).dimensions(width / 2 - btnW / 2, after, btnW, btnH).build());
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        MenuBackground.draw(this, context, mouseX, mouseY, delta);
        context.drawCenteredTextWithShadow(
            textRenderer,
            Text.literal("Texture packs PvP 1.21.11"),
            width / 2,
            48,
            UiTheme.accent()
        );
        if (!status.isEmpty()) {
            context.drawCenteredTextWithShadow(textRenderer, Text.literal(status), width / 2, height - 56, UiTheme.textDim());
        }
        super.render(context, mouseX, mouseY, delta);
    }
}
