package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.gui.theme.UiTheme;
import com.paraguacraft.pvp.resourcepack.CatalogPack;
import com.paraguacraft.pvp.resourcepack.CatalogLoader;
import com.paraguacraft.pvp.resourcepack.InstalledPack;
import com.paraguacraft.pvp.resourcepack.PackDropTarget;
import com.paraguacraft.pvp.resourcepack.ResourcePackManager;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiScreen;
import org.lwjgl.input.Keyboard;
import org.lwjgl.input.Mouse;

import javax.swing.JFileChooser;
import javax.swing.filechooser.FileNameExtensionFilter;
import java.io.File;
import java.io.IOException;
import java.util.List;

/**
 * Gestor de resource packs — descargas GitHub / Google Drive, importación y drag &amp; drop.
 */
public class GuiResourcePacks extends GuiScreen {

    private static final int CARD_W = 220;
    private static final int CARD_H = 88;
    private static final int ROW_H = 22;

    private List<InstalledPack> installed;
    private String statusText = "";
    private float progress = -1f;
    private boolean busy;
    private int scroll;
    private int catalogScroll;
    private CatalogPack[] featured = CatalogLoader.getFeatured();

    @Override
    public void initGui() {
        refreshList();
        PackDropTarget.register(new PackDropTarget.PackDropListener() {
            @Override
            public void onPackDropped(final File file) {
                mc.addScheduledTask(new Runnable() {
                    @Override
                    public void run() {
                        importLocal(file);
                    }
                });
            }
        });
    }

    @Override
    public void onGuiClosed() {
        PackDropTarget.unregister();
    }

    private void refreshList() {
        installed = ResourcePackManager.listInstalled();
    }

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        FontRenderer fr = this.fontRendererObj;
        drawRect(0, 0, width, height, 0x99000000);

        int panelX = Math.max(12, width / 2 - 400);
        int panelY = 28;
        int panelW = Math.min(width - 24, 800);
        int panelH = height - 56;
        Gui.drawRect(panelX, panelY, panelX + panelW, panelY + panelH, 0xCC0A0C14);
        Gui.drawRect(panelX, panelY, panelX + panelW, panelY + 1, 0x33FFFFFF);

        fr.drawStringWithShadow("RESOURCE PACKS", panelX + 16, panelY + 14, UiTheme.ACCENT);
        fr.drawStringWithShadow(
            "Catalogo PvP - GitHub / Google Drive - arrastra .zip - ESC volver",
            panelX + 16,
            panelY + 36,
            UiTheme.TEXT_DIM
        );

        int leftX = panelX + 16;
        int leftY = panelY + 58;
        int leftW = panelW / 2 - 24;
        fr.drawStringWithShadow("Destacados", leftX, leftY, UiTheme.TEXT);

        int listTop = leftY + 18;
        int listH = panelH - 130;
        Gui.drawRect(leftX, listTop, leftX + leftW, listTop + listH, 0x66000000);

        int visibleCards = Math.max(1, (listH - 8) / (CARD_H + 10));
        int maxCatalogScroll = Math.max(0, featured.length - visibleCards);
        if (catalogScroll > maxCatalogScroll) {
            catalogScroll = maxCatalogScroll;
        }

        int cardBaseY = listTop + 4;
        for (int i = 0; i < featured.length; i++) {
            int y = cardBaseY + (i - catalogScroll) * (CARD_H + 10);
            if (y + CARD_H < listTop || y > listTop + listH) {
                continue;
            }
            drawCatalogCard(featured[i], leftX + 4, y, mouseX, mouseY);
        }

        if (featured.length > visibleCards) {
            String scrollHint = (catalogScroll + 1) + "-" + Math.min(featured.length, catalogScroll + visibleCards)
                + " / " + featured.length + "  (rueda)";
            fr.drawStringWithShadow(scrollHint, leftX + leftW - fr.getStringWidth(scrollHint), leftY + 4, UiTheme.TEXT_DIM);
        }

        int rightX = panelX + panelW / 2 + 8;
        int rightY = leftY;
        int rightW = panelW / 2 - 24;
        fr.drawStringWithShadow("Instalados", rightX, rightY, UiTheme.TEXT);

        listTop = rightY + 18;
        listH = panelH - 130;
        Gui.drawRect(rightX, listTop, rightX + rightW, listTop + listH, 0x88000000);

        int maxRows = listH / ROW_H;
        for (int i = 0; i < Math.min(maxRows, installed.size() - scroll); i++) {
            InstalledPack pack = installed.get(i + scroll);
            int rowY = listTop + 4 + i * ROW_H;
            int nameColor = pack.active ? UiTheme.ACCENT : UiTheme.TEXT;
            fr.drawStringWithShadow(truncate(pack.displayName, 22), rightX + 8, rowY + 4, nameColor);
            String size = formatSize(pack.sizeBytes);
            fr.drawStringWithShadow(size, rightX + rightW - 120, rowY + 4, UiTheme.TEXT_DIM);
            if (pack.active) {
                fr.drawStringWithShadow("ACTIVO", rightX + rightW - 52, rowY + 4, 0xFF55FF88);
            }
        }
        if (installed.isEmpty()) {
            String empty = "Sin packs - descarga o importa uno";
            fr.drawStringWithShadow(empty, rightX + rightW / 2 - fr.getStringWidth(empty) / 2, listTop + listH / 2, UiTheme.TEXT_DIM);
        }

        int dropY = panelY + panelH - 72;
        boolean hoverDrop = mouseX >= leftX && mouseX <= leftX + leftW
            && mouseY >= dropY && mouseY <= dropY + 48;
        Gui.drawRect(leftX, dropY, leftX + leftW, dropY + 48, hoverDrop ? 0x4400E5FF : 0x2200E5FF);
        String dropLine = busy ? "Procesando..." : "Arrastra un .zip aca o usa Importar";
        fr.drawStringWithShadow(dropLine, leftX + leftW / 2 - fr.getStringWidth(dropLine) / 2, dropY + 10, UiTheme.TEXT);
        String dropHint = "Tambien podes soltar el archivo sobre la ventana del juego";
        fr.drawStringWithShadow(dropHint, leftX + leftW / 2 - fr.getStringWidth(dropHint) / 2, dropY + 28, UiTheme.TEXT_DIM);

        int btnY = panelY + panelH - 28;
        drawActionBtn("Importar .zip", leftX, btnY, 96, mouseX, mouseY, !busy);
        drawActionBtn("Quitar activo", rightX, btnY, 96, mouseX, mouseY, !busy);

        if (progress >= 0f) {
            int barX = panelX + 16;
            int barW = panelW - 32;
            int barY = panelY + panelH - 44;
            Gui.drawRect(barX, barY, barX + barW, barY + 8, 0x88000000);
            Gui.drawRect(barX, barY, barX + (int) (barW * progress), barY + 8, UiTheme.ACCENT);
            if (!statusText.isEmpty()) {
                fr.drawStringWithShadow(statusText, barX, barY - 10, UiTheme.TEXT_DIM);
            }
        }

        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    private void drawCatalogCard(CatalogPack pack, int x, int y, int mouseX, int mouseY) {
        FontRenderer fr = this.fontRendererObj;
        boolean hover = mouseX >= x && mouseX <= x + CARD_W && mouseY >= y && mouseY <= y + CARD_H;
        Gui.drawRect(x, y, x + CARD_W, y + CARD_H, hover ? 0xCC161A24 : 0xAA101218);
        Gui.drawRect(x + CARD_W - 36, y + 8, x + CARD_W - 8, y + 24, 0x3300E5FF);
        fr.drawStringWithShadow(pack.badge, x + CARD_W - 22 - fr.getStringWidth(pack.badge) / 2, y + 12, UiTheme.ACCENT);
        fr.drawStringWithShadow(pack.title, x + 10, y + 12, UiTheme.TEXT);
        fr.drawStringWithShadow(pack.subtitle, x + 10, y + 30, UiTheme.TEXT_DIM);
        int btnY = y + CARD_H - 24;
        boolean dlHover = mouseX >= x + 10 && mouseX <= x + CARD_W - 10 && mouseY >= btnY && mouseY <= btnY + 16;
        Gui.drawRect(x + 10, btnY, x + CARD_W - 10, btnY + 16, dlHover && !busy ? UiTheme.ACCENT : 0xFF226688);
        String dl = busy ? "..." : "Descargar e instalar";
        fr.drawStringWithShadow(dl, x + CARD_W / 2 - fr.getStringWidth(dl) / 2, btnY + 4, 0xFFFFFF);
    }

    private void drawActionBtn(String label, int x, int y, int w, int mx, int my, boolean enabled) {
        FontRenderer fr = this.fontRendererObj;
        boolean hover = enabled && mx >= x && mx <= x + w && my >= y && my <= y + 16;
        Gui.drawRect(x, y, x + w, y + 16, enabled ? (hover ? UiTheme.BTN_HOVER : UiTheme.BTN_BG) : 0x88444444);
        fr.drawStringWithShadow(label, x + w / 2 - fr.getStringWidth(label) / 2, y + 4, enabled ? UiTheme.TEXT : UiTheme.TEXT_DIM);
    }

    @Override
    protected void mouseClicked(int mouseX, int mouseY, int mouseButton) throws IOException {
        if (mouseButton != 0 || busy) {
            return;
        }
        int panelX = Math.max(12, width / 2 - 400);
        int panelY = 28;
        int panelW = Math.min(width - 24, 800);
        int panelH = height - 56;
        int leftX = panelX + 16;
        int leftY = panelY + 58;
        int leftW = panelW / 2 - 24;
        int rightX = panelX + panelW / 2 + 8;
        int rightW = panelW / 2 - 24;
        int listTop = leftY + 18;
        int listH = panelH - 130;
        int btnY = panelY + panelH - 28;

        if (hit(mouseX, mouseY, leftX, btnY, 96, 16)) {
            browseFile();
            return;
        }
        if (hit(mouseX, mouseY, rightX, btnY, 96, 16)) {
            ResourcePackManager.clearActivePack();
            refreshList();
            return;
        }

        int cardListTop = leftY + 18;
        int cardListH = panelH - 130;
        int cardBaseY = cardListTop + 4;
        for (int i = 0; i < featured.length; i++) {
            int cy = cardBaseY + (i - catalogScroll) * (CARD_H + 10);
            int dlY = cy + CARD_H - 24;
            if (hit(mouseX, mouseY, leftX + 14, dlY, CARD_W - 20, 16)) {
                startDownload(featured[i]);
                return;
            }
        }

        listTop = cardListTop;
        listH = cardListH;
        int maxRows = listH / ROW_H;
        for (int i = 0; i < Math.min(maxRows, installed.size() - scroll); i++) {
            int rowY = listTop + 4 + i * ROW_H;
            if (hit(mouseX, mouseY, rightX, rowY, rightW, ROW_H)) {
                InstalledPack pack = installed.get(i + scroll);
                ResourcePackManager.applyPack(pack.fileName);
                refreshList();
                return;
            }
        }
    }

    @Override
    public void handleMouseInput() throws IOException {
        super.handleMouseInput();
        int wheel = Mouse.getEventDWheel();
        if (wheel == 0) {
            return;
        }
        int panelX = Math.max(12, width / 2 - 400);
        int panelY = 28;
        int panelW = Math.min(width - 24, 800);
        int panelH = height - 56;
        int leftX = panelX + 16;
        int leftY = panelY + 58;
        int leftW = panelW / 2 - 24;
        int rightX = panelX + panelW / 2 + 8;
        int rightW = panelW / 2 - 24;
        int mx = Mouse.getEventX() * width / mc.displayWidth;
        int my = height - Mouse.getEventY() * height / mc.displayHeight - 1;

        int listTop = leftY + 18;
        int listH = panelH - 130;
        int visibleCards = Math.max(1, (listH - 8) / (CARD_H + 10));
        int maxCatalogScroll = Math.max(0, featured.length - visibleCards);

        if (mx >= leftX && mx <= leftX + leftW && my >= listTop && my <= listTop + listH) {
            catalogScroll = Math.max(0, Math.min(maxCatalogScroll, catalogScroll + (wheel > 0 ? -1 : 1)));
            return;
        }

        int installedTop = listTop;
        int maxRows = listH / ROW_H;
        int maxInstalledScroll = Math.max(0, installed.size() - maxRows);
        if (mx >= rightX && mx <= rightX + rightW && my >= installedTop && my <= installedTop + listH) {
            scroll = Math.max(0, Math.min(maxInstalledScroll, scroll + (wheel > 0 ? -1 : 1)));
        }
    }

    @Override
    protected void keyTyped(char typedChar, int keyCode) throws IOException {
        if (keyCode == Keyboard.KEY_ESCAPE) {
            mc.displayGuiScreen(null);
            return;
        }
        super.keyTyped(typedChar, keyCode);
    }

    private void browseFile() {
        JFileChooser chooser = new JFileChooser();
        chooser.setDialogTitle("Importar resource pack");
        chooser.setFileFilter(new FileNameExtensionFilter("Resource Pack (.zip)", "zip"));
        if (chooser.showOpenDialog(null) == JFileChooser.APPROVE_OPTION) {
            importLocal(chooser.getSelectedFile());
        }
    }

    private void importLocal(File file) {
        if (file == null || busy) {
            return;
        }
        try {
            busy = true;
            statusText = "Importando…";
            progress = 0.5f;
            String saved = ResourcePackManager.importFile(file);
            ResourcePackManager.applyPack(saved);
            refreshList();
            statusText = "Pack importado: " + saved;
            progress = 1f;
            PackDropTarget.notifyOk("Resource pack activado: " + saved);
        } catch (Exception e) {
            statusText = e.getMessage();
            progress = -1f;
            PackDropTarget.notifyError(statusText != null ? statusText : "Error al importar");
        } finally {
            busy = false;
        }
    }

    private void startDownload(final CatalogPack pack) {
        if (busy) {
            return;
        }
        busy = true;
        progress = 0.05f;
        statusText = "Iniciando descarga…";
        ResourcePackManager.downloadCatalogPack(pack, new ResourcePackManager.ProgressListener() {
            @Override
            public void onProgress(final String status, final float ratio) {
                mc.addScheduledTask(new Runnable() {
                    @Override
                    public void run() {
                        statusText = status;
                        progress = ratio;
                    }
                });
            }

            @Override
            public void onComplete(final String fileName) {
                mc.addScheduledTask(new Runnable() {
                    @Override
                    public void run() {
                        busy = false;
                        progress = 1f;
                        statusText = "Listo: " + fileName;
                        refreshList();
                        PackDropTarget.notifyOk("Pack instalado: " + fileName);
                    }
                });
            }

            @Override
            public void onError(final String message) {
                mc.addScheduledTask(new Runnable() {
                    @Override
                    public void run() {
                        busy = false;
                        progress = -1f;
                        statusText = message;
                        PackDropTarget.notifyError(message);
                    }
                });
            }
        });
    }

    private static boolean hit(int mx, int my, int x, int y, int w, int h) {
        return mx >= x && mx <= x + w && my >= y && my <= y + h;
    }

    private static String truncate(String s, int max) {
        if (s.length() <= max) {
            return s;
        }
        return s.substring(0, max - 1) + "…";
    }

    private static String formatSize(long bytes) {
        if (bytes < 1024) {
            return bytes + " B";
        }
        if (bytes < 1024 * 1024) {
            return (bytes / 1024) + " KB";
        }
        return String.format("%.1f MB", bytes / (1024.0 * 1024.0));
    }

    @Override
    public boolean doesGuiPauseGame() {
        return false;
    }
}
