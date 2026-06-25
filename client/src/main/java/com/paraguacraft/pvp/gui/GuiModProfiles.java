package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.core.ModProfileManager;
import com.paraguacraft.pvp.gui.theme.UiTheme;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiScreen;
import org.lwjgl.input.Keyboard;

import javax.swing.JFileChooser;
import javax.swing.filechooser.FileNameExtensionFilter;
import java.io.File;
import java.io.IOException;

/** Exportar / importar perfiles de configuración del cliente. */
public class GuiModProfiles extends GuiScreen {

    private String status = "Perfiles .json — ESC volver";

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        drawRect(0, 0, width, height, 0x99000000);
        int px = width / 2 - 160;
        int py = height / 2 - 60;
        Gui.drawRect(px, py, px + 320, py + 120, 0xCC0A0C14);
        FontRenderer fr = fontRendererObj;
        fr.drawStringWithShadow("PERFILES DE MODS", px + 16, py + 12, UiTheme.ACCENT);
        fr.drawStringWithShadow(status, px + 16, py + 36, UiTheme.TEXT_DIM);
        drawBtn("Exportar perfil", px + 16, py + 64, 130, mouseX, mouseY);
        drawBtn("Importar perfil", px + 174, py + 64, 130, mouseX, mouseY);
        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    private void drawBtn(String label, int x, int y, int w, int mx, int my) {
        boolean hover = mx >= x && mx <= x + w && my >= y && my <= y + 18;
        Gui.drawRect(x, y, x + w, y + 18, hover ? UiTheme.BTN_HOVER : UiTheme.BTN_BG);
        fontRendererObj.drawStringWithShadow(label, x + w / 2 - fontRendererObj.getStringWidth(label) / 2, y + 5, UiTheme.TEXT);
    }

    @Override
    protected void mouseClicked(int mouseX, int mouseY, int mouseButton) throws IOException {
        if (mouseButton != 0) {
            return;
        }
        int px = width / 2 - 160;
        int py = height / 2 - 60;
        if (hit(mouseX, mouseY, px + 16, py + 64, 130, 18)) {
            exportProfile();
        } else if (hit(mouseX, mouseY, px + 174, py + 64, 130, 18)) {
            importProfile();
        }
    }

    private void exportProfile() {
        JFileChooser chooser = new JFileChooser(ModProfileManager.profilesDir());
        chooser.setSelectedFile(new File("mi-perfil.json"));
        chooser.setFileFilter(new FileNameExtensionFilter("Perfil Paraguacraft (.json)", "json"));
        if (chooser.showSaveDialog(null) == JFileChooser.APPROVE_OPTION) {
            try {
                File f = chooser.getSelectedFile();
                if (!f.getName().toLowerCase().endsWith(".json")) {
                    f = new File(f.getParentFile(), f.getName() + ".json");
                }
                ModProfileManager.exportTo(f);
                status = "Exportado: " + f.getName();
            } catch (Exception e) {
                status = e.getMessage() != null ? e.getMessage() : "Error al exportar";
            }
        }
    }

    private void importProfile() {
        JFileChooser chooser = new JFileChooser(ModProfileManager.profilesDir());
        chooser.setFileFilter(new FileNameExtensionFilter("Perfil Paraguacraft (.json)", "json"));
        if (chooser.showOpenDialog(null) == JFileChooser.APPROVE_OPTION) {
            try {
                ModProfileManager.importFrom(chooser.getSelectedFile());
                status = "Perfil importado";
            } catch (Exception e) {
                status = e.getMessage() != null ? e.getMessage() : "Error al importar";
            }
        }
    }

    private static boolean hit(int mx, int my, int x, int y, int w, int h) {
        return mx >= x && mx <= x + w && my >= y && my <= y + h;
    }

    @Override
    protected void keyTyped(char typedChar, int keyCode) throws IOException {
        if (keyCode == Keyboard.KEY_ESCAPE) {
            mc.displayGuiScreen(new GuiParaguaMenu());
            return;
        }
        super.keyTyped(typedChar, keyCode);
    }

    @Override
    public boolean doesGuiPauseGame() {
        return false;
    }
}
