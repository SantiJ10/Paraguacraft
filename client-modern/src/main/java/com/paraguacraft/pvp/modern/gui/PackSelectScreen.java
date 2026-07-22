package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.ParaguacraftPvPModern;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import com.paraguacraft.pvp.modern.resourcepack.CatalogLoader;
import com.paraguacraft.pvp.modern.resourcepack.CatalogPack;
import com.paraguacraft.pvp.modern.resourcepack.ResourcePackService;
import net.minecraft.client.gl.RenderPipelines;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;
import net.minecraft.util.Identifier;

import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;

/** Selector de texture packs: principal + secundario opcional. */
public class PackSelectScreen extends ParaguacraftScreen {

    private static final Identifier PACK_ICON = Identifier.of(ParaguacraftPvPModern.MOD_ID, "textures/gui/mod_icon.png");

    private String status = "";
    private final List<CatalogPack> packs = new ArrayList<>();

    public PackSelectScreen(Screen parent) {
        super(Text.literal("Texture packs PvP"), parent);
    }

    @Override
    protected void init() {
        packs.clear();
        Set<String> seen = new LinkedHashSet<>();
        for (CatalogPack pack : CatalogLoader.getFeatured()) {
            if (seen.add(pack.fileName)) {
                packs.add(pack);
            }
        }
        for (String fileName : ResourcePackService.listInstalledZipNames()) {
            if (seen.add(fileName)) {
                packs.add(new CatalogPack(
                    fileName,
                    fileName.replace(".zip", ""),
                    "",
                    "",
                    fileName,
                    "",
                    ""
                ));
            }
        }
        rebuild();
    }

    private void rebuild() {
        clearChildren();
        int btnW = 280;
        int btnH = 20;
        int startY = 88;
        int gap = 52;

        addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, startY - 28, btnW, btnH,
            Text.literal("Aplicar packs guardados"), () -> {
                ResourcePackService.applySessionPacks();
                status = "Packs de sesion aplicados";
            }));

        for (int i = 0; i < packs.size(); i++) {
            CatalogPack pack = packs.get(i);
            int y = startY + i * gap;
            boolean installed = ResourcePackService.isInstalled(pack.fileName);
            if (!installed && pack.downloadUrl != null && !pack.downloadUrl.isEmpty()) {
                addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, y, btnW, btnH,
                    Text.literal(pack.title + " ↓ Descargar"), () -> download(pack)));
                continue;
            }
            if (!installed) {
                continue;
            }
            boolean isPrimary = isPrimary(pack.fileName);
            boolean isSecondary = isSecondary(pack.fileName);
            addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2 - 2, y + 14, btnW / 2 - 4, btnH,
                Text.literal((isPrimary ? "★ " : "") + "Principal"), () -> setPrimary(pack)));
            addDrawableChild(FlatMenuButton.create(width / 2 + 4, y + 14, btnW / 2 - 4, btnH,
                Text.literal((isSecondary ? "+ " : "") + "Secundario"), () -> setSecondary(pack)));
            if (isSecondary) {
                addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, y + 14 + btnH + 2, btnW, btnH,
                    Text.literal("Quitar secundario"), () -> {
                        ResourcePackService.setSecondaryPack("");
                        status = "Pack secundario quitado";
                        rebuild();
                    }));
            }
        }

        int after = startY + packs.size() * gap + 8;
        addDrawableChild(FlatMenuButton.create(width / 2 - btnW / 2, after, btnW, btnH,
            Text.literal("Volver"), () -> client.setScreen(parent)));
    }

    private boolean isPrimary(String fileName) {
        String primary = ModernConfig.selectedResourcePack;
        if (primary == null || primary.isBlank()) {
            primary = ResourcePackService.OFFICIAL_PACK;
        }
        return fileName.equalsIgnoreCase(primary);
    }

    private boolean isSecondary(String fileName) {
        return ModernConfig.secondaryResourcePack != null
            && fileName.equalsIgnoreCase(ModernConfig.secondaryResourcePack);
    }

    private void setPrimary(CatalogPack pack) {
        if (!ResourcePackService.OFFICIAL_PACK.equalsIgnoreCase(pack.fileName)) {
            if (ResourcePackService.isInstalled(ResourcePackService.OFFICIAL_PACK)) {
                setSecondary(pack);
                status = pack.title + " aplicado como secundario (el oficial es siempre principal)";
                return;
            }
        }
        ModernConfig.selectedResourcePack = pack.fileName;
        ModernConfig.save();
        ResourcePackService.applySessionPacks();
        status = "Principal: " + pack.title;
        rebuild();
    }

    private void setSecondary(CatalogPack pack) {
        if (isPrimary(pack.fileName)) {
            status = "El principal no puede ser secundario";
            return;
        }
        ResourcePackService.setSecondaryPack(pack.fileName);
        status = "Secundario: " + pack.title;
        rebuild();
    }

    private void download(CatalogPack pack) {
        status = "Descargando " + pack.title + "…";
        ResourcePackService.download(pack, new ResourcePackService.ProgressListener() {
            @Override
            public void onProgress(String s, float ratio) {
                status = s;
            }

            @Override
            public void onComplete(String fileName) {
                status = "Listo: " + pack.title;
                client.execute(PackSelectScreen.this::init);
            }

            @Override
            public void onError(String message) {
                status = "Error: " + message;
            }
        });
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        super.render(context, mouseX, mouseY, delta);
        context.drawCenteredTextWithShadow(
            textRenderer,
            Text.literal("Texture packs PvP 1.21.11"),
            width / 2,
            36,
            UiTheme.accent()
        );
        String primary = ModernConfig.selectedResourcePack;
        if (primary == null || primary.isBlank()) {
            primary = ResourcePackService.OFFICIAL_PACK;
        }
        String secondary = ModernConfig.secondaryResourcePack == null || ModernConfig.secondaryResourcePack.isBlank()
            ? "(ninguno)"
            : ModernConfig.secondaryResourcePack;
        context.drawCenteredTextWithShadow(
            textRenderer,
            Text.literal("Principal: " + primary),
            width / 2,
            50,
            UiTheme.textDim()
        );
        context.drawCenteredTextWithShadow(
            textRenderer,
            Text.literal("Secundario: " + secondary),
            width / 2,
            62,
            UiTheme.textDim()
        );

        int listX = width / 2 - 150;
        int startY = 88;
        int gap = 52;
        for (int i = 0; i < packs.size(); i++) {
            CatalogPack pack = packs.get(i);
            int y = startY + i * gap;
            context.drawCenteredTextWithShadow(textRenderer, Text.literal(pack.title), width / 2, y, UiTheme.textDim());
            if ("paraguacraft-pvp".equals(pack.id)) {
                int iconSize = 32;
                context.drawTexture(RenderPipelines.GUI_TEXTURED, PACK_ICON,
                    listX - iconSize - 8, y - 5, 0f, 0f, iconSize, iconSize, iconSize, iconSize);
            }
        }
        if (!status.isEmpty()) {
            context.drawCenteredTextWithShadow(textRenderer, Text.literal(status), width / 2, height - 56, UiTheme.textDim());
        }
    }
}
