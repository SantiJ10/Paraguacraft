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
        String state();
        int stateColor();
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
            Gui.drawRect(px + 12, rowY, px + 308, rowY + 20, 0x44000000);
            fr.drawStringWithShadow(rows[i].label(), px + 20, rowY + 6, UiTheme.TEXT);
            String state = rows[i].state();
            fr.drawStringWithShadow(state, px + 280 - fr.getStringWidth(state), rowY + 6, rows[i].stateColor());
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
            row("Mostrar durabilidad numerica", new BoolGet() { public boolean get() { return ModConfig.showArmorDurability; } }, new BoolSet() { public void set(boolean v) { ModConfig.showArmorDurability = v; } }),
            row("Alerta de durabilidad baja", new BoolGet() { public boolean get() { return ModConfig.armorDurabilityAlert; } }, new BoolSet() { public void set(boolean v) { ModConfig.armorDurabilityAlert = v; } }),
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
            row("Mostrar nombres", new BoolGet() { public boolean get() { return ModConfig.showItemNames; } }, new BoolSet() { public void set(boolean v) { ModConfig.showItemNames = v; } }),
            row("Contador de bloques", new BoolGet() { public boolean get() { return ModConfig.showBlockCount; } }, new BoolSet() { public void set(boolean v) { ModConfig.showBlockCount = v; } }),
            row("Perfiles auto por modo", new BoolGet() { public boolean get() { return ModConfig.autoGameModeProfiles; } }, new BoolSet() { public void set(boolean v) { ModConfig.autoGameModeProfiles = v; ModConfig.autoBedwarsHud = v; } }),
            row("Fondo transparente", new BoolGet() { public boolean get() { return ModConfig.bwResTransparentBg; } }, new BoolSet() { public void set(boolean v) { ModConfig.bwResTransparentBg = v; } }),
        });
    }

    public static GuiSubmodOptions keystrokes() {
        return new GuiSubmodOptions("Teclas", new Row[] {
            row("Mostrar teclas WASD + espacio", new BoolGet() { public boolean get() { return ModConfig.showKeystrokes; } }, new BoolSet() { public void set(boolean v) { ModConfig.showKeystrokes = v; } }),
            row("Mostrar LMB / RMB", new BoolGet() { public boolean get() { return ModConfig.showKeystrokesMouse; } }, new BoolSet() { public void set(boolean v) { ModConfig.showKeystrokesMouse = v; } }),
        });
    }

    public static GuiSubmodOptions nametag() {
        return new GuiSubmodOptions("Nametag / UI", new Row[] {
            row("Logo en nametag local", new BoolGet() { public boolean get() { return ModConfig.showNametagLogo; } }, new BoolSet() { public void set(boolean v) { ModConfig.showNametagLogo = v; } }),
            row("Logo en otros jugadores", new BoolGet() { public boolean get() { return ModConfig.showNametagLogoOthers; } }, new BoolSet() { public void set(boolean v) { ModConfig.showNametagLogoOthers = v; } }),
            row("Salud en nametag", new BoolGet() { public boolean get() { return ModConfig.showNametagHealth; } }, new BoolSet() { public void set(boolean v) { ModConfig.showNametagHealth = v; } }),
            row("Nombre y salud en inventario", new BoolGet() { public boolean get() { return ModConfig.showInventoryTags; } }, new BoolSet() { public void set(boolean v) { ModConfig.showInventoryTags = v; } }),
            row("Watermark en contenedores", new BoolGet() { public boolean get() { return ModConfig.showWatermark; } }, new BoolSet() { public void set(boolean v) { ModConfig.showWatermark = v; } }),
        });
    }

    public static GuiSubmodOptions items() {
        return new GuiSubmodOptions("Item Tracker", new Row[] {
            row("Lista 2D (HUD)", new BoolGet() { public boolean get() { return ModConfig.itemTracker2d; } }, new BoolSet() { public void set(boolean v) { ModConfig.itemTracker2d = v; } }),
            row("Etiquetas 3D en el mundo", new BoolGet() { public boolean get() { return ModConfig.itemTracker3d; } }, new BoolSet() { public void set(boolean v) { ModConfig.itemTracker3d = v; } }),
        });
    }

    public static GuiSubmodOptions tnt() {
        return new GuiSubmodOptions("Cuenta TNT", new Row[] {
            row("Mostrar countdown", new BoolGet() { public boolean get() { return ModConfig.showTntCountdown; } }, new BoolSet() { public void set(boolean v) { ModConfig.showTntCountdown = v; } }),
            row("Compensar ping", new BoolGet() { public boolean get() { return ModConfig.tntPingCompensate; } }, new BoolSet() { public void set(boolean v) { ModConfig.tntPingCompensate = v; } }),
            cycle("Fuse (Auto / 80 / 60 / 52 / 40)", new LabelGet() { public String get() { return ModConfig.tntFuseLabel(); } }, new Runnable() { public void run() { ModConfig.cycleTntFuse(); } }),
        });
    }

    public static GuiSubmodOptions chat() {
        return new GuiSubmodOptions("Chat", new Row[] {
            row("Historial ilimitado", new BoolGet() { public boolean get() { return ModConfig.chatUnlimited; } }, new BoolSet() { public void set(boolean v) { ModConfig.chatUnlimited = v; } }),
            row("Sombra de texto", new BoolGet() { public boolean get() { return ModConfig.chatTextShadow; } }, new BoolSet() { public void set(boolean v) { ModConfig.chatTextShadow = v; } }),
            row("Resaltar tu nombre", new BoolGet() { public boolean get() { return ModConfig.chatHighlightName; } }, new BoolSet() { public void set(boolean v) { ModConfig.chatHighlightName = v; } }),
        });
    }

    public static GuiSubmodOptions tab() {
        return new GuiSubmodOptions("Editor de Tab", new Row[] {
            row("Editor activo", new BoolGet() { public boolean get() { return ModConfig.tabEditor; } }, new BoolSet() { public void set(boolean v) { ModConfig.tabEditor = v; } }),
            row("Ping como numero", new BoolGet() { public boolean get() { return ModConfig.tabPingNumbers; } }, new BoolSet() { public void set(boolean v) { ModConfig.tabPingNumbers = v; } }),
            row("Ocultar NPCs", new BoolGet() { public boolean get() { return ModConfig.tabHideNpcs; } }, new BoolSet() { public void set(boolean v) { ModConfig.tabHideNpcs = v; } }),
            row("Ocultar ping > 500", new BoolGet() { public boolean get() { return ModConfig.tabHideHighPing; } }, new BoolSet() { public void set(boolean v) { ModConfig.tabHideHighPing = v; } }),
        });
    }

    public static GuiSubmodOptions hitColor() {
        return new GuiSubmodOptions("Color de golpe", new Row[] {
            row("Color de golpe activo", new BoolGet() { public boolean get() { return ModConfig.hitColorEnabled; } }, new BoolSet() { public void set(boolean v) { ModConfig.hitColorEnabled = v; } }),
            cycle("Preset", new LabelGet() { public String get() { return ModConfig.hitColorLabel(); } }, new Runnable() { public void run() { ModConfig.cycleHitColor(); } }),
        });
    }

    public static GuiSubmodOptions particles() {
        return new GuiSubmodOptions("Particulas", new Row[] {
            cycle("Modo", new LabelGet() { public String get() { return PerformanceConfig.particleMode.getLabel(); } }, new Runnable() { public void run() { PerformanceConfig.cycleParticleMode(); } }),
            row("Ocultar explosiones", new BoolGet() { public boolean get() { return PerformanceConfig.hideExplosionParticles; } }, new BoolSet() { public void set(boolean v) { PerformanceConfig.hideExplosionParticles = v; } }),
            row("Ocultar particulas de pocion", new BoolGet() { public boolean get() { return PerformanceConfig.hidePotionParticles; } }, new BoolSet() { public void set(boolean v) { PerformanceConfig.hidePotionParticles = v; } }),
        });
    }

    public static GuiSubmodOptions reach() {
        return new GuiSubmodOptions("Reach display", new Row[] {
            row("Mostrar alcance", new BoolGet() { public boolean get() { return ModConfig.reachDisplay; } }, new BoolSet() { public void set(boolean v) { ModConfig.reachDisplay = v; } }),
            row("Mantener ultimo golpe (4s)", new BoolGet() { public boolean get() { return ModConfig.reachPersist; } }, new BoolSet() { public void set(boolean v) { ModConfig.reachPersist = v; } }),
        });
    }

    public static GuiSubmodOptions guiScale() {
        return new GuiSubmodOptions("Escala GUI", new Row[] {
            cycle("Hotbar", new LabelGet() { public String get() { return ModConfig.hotbarScale + "%"; } }, new Runnable() { public void run() { ModConfig.cycleHotbarScale(); } }),
            cycle("Inventario", new LabelGet() { public String get() { return ModConfig.inventoryScale + "%"; } }, new Runnable() { public void run() { ModConfig.cycleInventoryScale(); } }),
        });
    }

    public static GuiSubmodOptions packHud() {
        return new GuiSubmodOptions("Visualizacion de pack", new Row[] {
            row("Mostrar pack actual en HUD", new BoolGet() { public boolean get() { return ModConfig.showPackHud; } }, new BoolSet() { public void set(boolean v) { ModConfig.showPackHud = v; } }),
        });
    }

    public static GuiSubmodOptions heightLimit() {
        return new GuiSubmodOptions("Limite de altura", new Row[] {
            row("Mostrar altura / techo", new BoolGet() { public boolean get() { return ModConfig.showHeightLimit; } }, new BoolSet() { public void set(boolean v) { ModConfig.showHeightLimit = v; } }),
            cycle("Techo (Auto / 256 / 320)", new LabelGet() { public String get() { return ModConfig.heightLimitLabel(); } }, new Runnable() { public void run() { ModConfig.cycleHeightLimit(); } }),
        });
    }

    private static Row row(final String label, final BoolGet get, final BoolSet set) {
        return new Row() {
            public String label() { return label; }
            public String state() { return get.get() ? "ON" : "OFF"; }
            public int stateColor() { return get.get() ? 0xFF22CC66 : 0xFFCC4444; }
            public void toggle() { set.set(!get.get()); }
        };
    }

    private static Row cycle(final String label, final LabelGet get, final Runnable next) {
        return new Row() {
            public String label() { return label; }
            public String state() { return get.get(); }
            public int stateColor() { return UiTheme.ACCENT; }
            public void toggle() { next.run(); }
        };
    }

    private interface BoolGet { boolean get(); }
    private interface BoolSet { void set(boolean v); }
    private interface LabelGet { String get(); }
}
