package com.paraguacraft.pvp.gui;

import com.paraguacraft.pvp.core.PerformanceConfig;
import com.paraguacraft.pvp.gui.theme.UiTheme;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.gui.FontRenderer;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.GuiScreen;
import org.lwjgl.input.Keyboard;

import java.io.IOException;

/** Submods agrupados (estilo Musica / Scoreboard). */
public class GuiSubmodOptions extends GuiScreen {

    public interface Row {
        String label();
        boolean value();
        void toggle();
    }

    private final String title;
    private final Row[] rows;

    public GuiSubmodOptions(String title, Row[] rows) {
        this.title = title;
        this.rows = rows;
    }

    @Override
    public void drawScreen(int mouseX, int mouseY, float partialTicks) {
        drawRect(0, 0, width, height, 0x99000000);
        int h = 56 + rows.length * 28;
        int px = width / 2 - 160;
        int py = Math.max(20, height / 2 - h / 2);
        Gui.drawRect(px, py, px + 320, py + h, 0xCC0A0C14);
        FontRenderer fr = fontRendererObj;
        fr.drawStringWithShadow(title, px + 16, py + 12, UiTheme.ACCENT);
        for (int i = 0; i < rows.length; i++) {
            int rowY = py + 40 + i * 28;
            boolean on = rows[i].value();
            Gui.drawRect(px + 12, rowY, px + 308, rowY + 20, 0x44000000);
            fr.drawStringWithShadow(rows[i].label(), px + 20, rowY + 6, UiTheme.TEXT);
            String state = on ? "ON" : "OFF";
            fr.drawStringWithShadow(state, px + 280 - fr.getStringWidth(state), rowY + 6, on ? 0xFF22CC66 : 0xFFCC4444);
        }
        super.drawScreen(mouseX, mouseY, partialTicks);
    }

    @Override
    protected void mouseClicked(int mouseX, int mouseY, int mouseButton) throws IOException {
        if (mouseButton != 0) {
            return;
        }
        int h = 56 + rows.length * 28;
        int px = width / 2 - 160;
        int py = Math.max(20, height / 2 - h / 2);
        for (int i = 0; i < rows.length; i++) {
            int rowY = py + 40 + i * 28;
            if (mouseX >= px + 12 && mouseX <= px + 308 && mouseY >= rowY && mouseY <= rowY + 20) {
                rows[i].toggle();
                ModConfig.save();
                return;
            }
        }
    }

    @Override
    protected void keyTyped(char typedChar, int keyCode) throws IOException {
        if (keyCode == Keyboard.KEY_ESCAPE) {
            mc.displayGuiScreen(new GuiParaguaMenu());
        }
    }

    @Override
    public boolean doesGuiPauseGame() {
        return false;
    }

    public static GuiSubmodOptions armor() {
        return new GuiSubmodOptions("Armadura HUD", new Row[] {
            row("Mostrar iconos de armadura", new BoolGet() { public boolean get() { return ModConfig.showArmor; } }, new BoolSet() { public void set(boolean v) { ModConfig.showArmor = v; } }),
            row("Mostrar % de durabilidad", new BoolGet() { public boolean get() { return ModConfig.showArmorPercentage; } }, new BoolSet() { public void set(boolean v) { ModConfig.showArmorPercentage = v; } }),
        });
    }

    public static GuiSubmodOptions fps() {
        return new GuiSubmodOptions("FPS", new Row[] {
            row("Mostrar FPS", new BoolGet() { public boolean get() { return ModConfig.showFPS; } }, new BoolSet() { public void set(boolean v) { ModConfig.showFPS = v; } }),
            row("Limitar FPS en menu/idle", new BoolGet() { public boolean get() { return PerformanceConfig.reduceFpsWhenMinimized; } }, new BoolSet() { public void set(boolean v) { PerformanceConfig.reduceFpsWhenMinimized = v; } }),
            row("Bajar FPS sin foco", new BoolGet() { public boolean get() { return PerformanceConfig.reduceFpsWhenUnfocused; } }, new BoolSet() { public void set(boolean v) { PerformanceConfig.reduceFpsWhenUnfocused = v; } }),
        });
    }

    public static GuiSubmodOptions entity() {
        return new GuiSubmodOptions("Entity / cull", new Row[] {
            row("Entity cull", new BoolGet() { public boolean get() { return PerformanceConfig.entityCull; } }, new BoolSet() { public void set(boolean v) { PerformanceConfig.entityCull = v; } }),
            row("Nametag cull", new BoolGet() { public boolean get() { return PerformanceConfig.nametagCull; } }, new BoolSet() { public void set(boolean v) { PerformanceConfig.nametagCull = v; } }),
            row("Block entity cull", new BoolGet() { public boolean get() { return PerformanceConfig.blockEntityCull; } }, new BoolSet() { public void set(boolean v) { PerformanceConfig.blockEntityCull = v; } }),
            row("Anim freeze lejos", new BoolGet() { public boolean get() { return PerformanceConfig.entityAnimCull; } }, new BoolSet() { public void set(boolean v) { PerformanceConfig.entityAnimCull = v; } }),
            row("Armor stand cull", new BoolGet() { public boolean get() { return PerformanceConfig.armorStandCull; } }, new BoolSet() { public void set(boolean v) { PerformanceConfig.armorStandCull = v; } }),
            row("Item frame cull", new BoolGet() { public boolean get() { return PerformanceConfig.itemFrameCull; } }, new BoolSet() { public void set(boolean v) { PerformanceConfig.itemFrameCull = v; } }),
            row("Nametag LOD", new BoolGet() { public boolean get() { return PerformanceConfig.nametagLod; } }, new BoolSet() { public void set(boolean v) { PerformanceConfig.nametagLod = v; } }),
        });
    }

    public static GuiSubmodOptions bedwars() {
        return new GuiSubmodOptions("BedWars", new Row[] {
            row("HUD recursos BedWars", new BoolGet() { public boolean get() { return ModConfig.showBedwarsResources; } }, new BoolSet() { public void set(boolean v) { ModConfig.showBedwarsResources = v; } }),
            row("Fondo transparente", new BoolGet() { public boolean get() { return ModConfig.bwResTransparentBg; } }, new BoolSet() { public void set(boolean v) { ModConfig.bwResTransparentBg = v; } }),
        });
    }

    private static Row row(final String label, final BoolGet get, final BoolSet set) {
        return new Row() {
            public String label() { return label; }
            public boolean value() { return get.get(); }
            public void toggle() { set.set(!get.get()); }
        };
    }

    private interface BoolGet { boolean get(); }
    private interface BoolSet { void set(boolean v); }
}
