package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.core.ModProfileManager;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.TextFieldWidget;
import net.minecraft.text.Text;

import java.io.File;
import java.util.List;

/** Export/import de perfiles de mods (paridad con `ModProfileManager` 1.8.9). */
public class ModProfilesScreen extends ParaguacraftScreen {

    private TextFieldWidget nameField;
    private List<File> profiles = List.of();
    private String status = "";

    public ModProfilesScreen(Screen parent) {
        super(Text.literal("Perfiles de mods"), parent);
    }

    @Override
    protected void init() {
        profiles = ModProfileManager.listProfiles();
        int panelW = Math.min(360, width - 40);
        int cx = width / 2;
        int y = height / 2 - 110;

        nameField = new TextFieldWidget(textRenderer, cx - panelW / 2, y + 24, panelW - 90, 20, Text.literal("Nombre"));
        nameField.setMaxLength(48);
        nameField.setText("perfil");
        addDrawableChild(nameField);

        addDrawableChild(FlatMenuButton.create(cx + panelW / 2 - 86, y + 24, 86, 20,
            Text.literal("Guardar"), this::saveProfile));

        int listY = y + 54;
        int max = Math.min(6, profiles.size());
        for (int i = 0; i < max; i++) {
            File f = profiles.get(i);
            int fy = listY + i * 24;
            addDrawableChild(FlatMenuButton.create(cx - panelW / 2, fy, panelW - 90, 20,
                Text.literal(stripExt(f.getName())), () -> loadProfile(f)));
            addDrawableChild(FlatMenuButton.create(cx + panelW / 2 - 86, fy, 86, 20,
                Text.literal("Borrar"), () -> deleteProfile(f)));
        }

        addDrawableChild(FlatMenuButton.create(cx - panelW / 2, height / 2 + 110, panelW, 22,
            Text.literal("Volver"), () -> client.setScreen(parent)));
    }

    private void saveProfile() {
        String name = nameField.getText().trim();
        if (name.isEmpty()) {
            status = "Poné un nombre primero.";
            return;
        }
        try {
            File dest = new File(ModProfileManager.profilesDir(), name + ".json");
            ModProfileManager.exportTo(dest);
            status = "Guardado: " + name;
            clearChildren();
            init();
        } catch (Exception e) {
            status = "Error al guardar: " + e.getMessage();
        }
    }

    private void loadProfile(File f) {
        try {
            ModProfileManager.importFrom(f);
            status = "Perfil aplicado: " + stripExt(f.getName());
        } catch (Exception e) {
            status = "Error al cargar: " + e.getMessage();
        }
    }

    private void deleteProfile(File f) {
        if (f.delete()) {
            status = "Eliminado: " + stripExt(f.getName());
            clearChildren();
            init();
        }
    }

    private static String stripExt(String name) {
        int dot = name.lastIndexOf('.');
        return dot > 0 ? name.substring(0, dot) : name;
    }

    @Override
    public void render(DrawContext ctx, int mouseX, int mouseY, float delta) {
        super.render(ctx, mouseX, mouseY, delta);
        int panelW = Math.min(360, width - 40);
        int cx = width / 2;
        int y = height / 2 - 110;
        ctx.fill(cx - panelW / 2 - 8, y - 8, cx + panelW / 2 + 8, height / 2 + 140, 0xCC0A0C14);
        ctx.drawCenteredTextWithShadow(textRenderer, Text.literal("Perfiles de mods"), cx, y, UiTheme.accent());
        if (profiles.isEmpty()) {
            ctx.drawCenteredTextWithShadow(textRenderer, Text.literal("Sin perfiles guardados todavía"), cx, y + 54, UiTheme.textDim());
        }
        if (!status.isEmpty()) {
            ctx.drawCenteredTextWithShadow(textRenderer, Text.literal(status), cx, height / 2 + 96, UiTheme.textDim());
        }
    }
}
