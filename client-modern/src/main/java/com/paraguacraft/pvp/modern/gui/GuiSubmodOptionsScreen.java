package com.paraguacraft.pvp.modern.gui;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.core.PerformanceConfig;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.gui.Click;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;

import java.util.function.BooleanSupplier;
import java.util.function.Consumer;
import java.util.function.Supplier;

/** Submods agrupados (Armadura, FPS, Entity, BedWars, Chat, Scoreboard). */
public class GuiSubmodOptionsScreen extends ParaguacraftScreen {

    public record Row(String label, BooleanSupplier getter, Consumer<Boolean> setter, Supplier<String> state) {
        public Row(String label, BooleanSupplier getter, Consumer<Boolean> setter) {
            this(label, getter, setter, null);
        }
    }

    private final Row[] rows;

    public GuiSubmodOptionsScreen(Screen parent, String title, Row[] rows) {
        super(Text.literal(title), parent);
        this.rows = rows;
    }

    @Override
    public void render(DrawContext ctx, int mouseX, int mouseY, float delta) {
        renderBackground(ctx, mouseX, mouseY, delta);
        int h = 56 + rows.length * 32;
        int px = width / 2 - 160;
        int py = Math.max(20, height / 2 - h / 2);
        ctx.fill(px, py, px + 320, py + h, 0xCC0A0C14);
        ctx.drawText(textRenderer, title, px + 16, py + 12, UiTheme.accent(), true);
        for (int i = 0; i < rows.length; i++) {
            int rowY = py + 44 + i * 32;
            ctx.fill(px + 12, rowY, px + 308, rowY + 22, 0x44000000);
            boolean on = rows[i].getter.getAsBoolean();
            ctx.drawText(textRenderer, Text.literal(rows[i].label), px + 20, rowY + 7, UiTheme.TEXT, true);
            String value = rows[i].state() != null ? rows[i].state().get() : (on ? "ON" : "OFF");
            int color = rows[i].state() != null ? UiTheme.accent() : (on ? 0xFF22CC66 : 0xFFCC4444);
            ctx.drawText(textRenderer, Text.literal(value), px + 288 - textRenderer.getWidth(value), rowY + 7, color, true);
        }
        super.render(ctx, mouseX, mouseY, delta);
    }

    @Override
    public boolean mouseClicked(Click click, boolean doubled) {
        if (click.buttonInfo().button() != 0) {
            return super.mouseClicked(click, doubled);
        }
        int h = 56 + rows.length * 32;
        int px = width / 2 - 160;
        int py = Math.max(20, height / 2 - h / 2);
        for (int i = 0; i < rows.length; i++) {
            int rowY = py + 44 + i * 32;
            if (click.x() >= px + 12 && click.x() <= px + 308 && click.y() >= rowY && click.y() <= rowY + 22) {
                if (rows[i].state() != null) {
                    rows[i].setter.accept(true);
                } else {
                    rows[i].setter.accept(!rows[i].getter.getAsBoolean());
                }
                ModernConfig.save();
                return true;
            }
        }
        return super.mouseClicked(click, doubled);
    }

    public static GuiSubmodOptionsScreen armor(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "Armadura HUD", new Row[] {
            new Row("Mostrar iconos", () -> ModernConfig.showArmor, v -> ModernConfig.showArmor = v),
            new Row("Mostrar % durabilidad", () -> ModernConfig.showArmorPercentage, v -> ModernConfig.showArmorPercentage = v),
            new Row("Durabilidad numerica", () -> ModernConfig.showArmorDurability, v -> ModernConfig.showArmorDurability = v),
            new Row("Alerta durabilidad baja", () -> ModernConfig.armorDurabilityAlert, v -> ModernConfig.armorDurabilityAlert = v),
        });
    }

    public static GuiSubmodOptionsScreen keystrokes(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "Keystrokes", new Row[] {
            new Row("Mostrar HUD (WASD + espacio)", () -> ModernConfig.showKeystrokes, v -> ModernConfig.showKeystrokes = v),
            new Row("Mostrar LMB / RMB", () -> ModernConfig.showKeystrokesMouse, v -> ModernConfig.showKeystrokesMouse = v),
        });
    }

    public static GuiSubmodOptionsScreen cosmetics(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "Cosmeticos / nametags", new Row[] {
            new Row("Vida en nametag", () -> ModernConfig.showNametagHealth, v -> ModernConfig.showNametagHealth = v),
            new Row("Logo propio", () -> ModernConfig.showNametagLogo, v -> ModernConfig.showNametagLogo = v),
            new Row("Logo en rivales", () -> ModernConfig.showNametagLogoOthers, v -> ModernConfig.showNametagLogoOthers = v),
            new Row("Overlay inventario", () -> ModernConfig.showInventoryTags, v -> ModernConfig.showInventoryTags = v),
            new Row("Watermark GUI", () -> ModernConfig.showWatermark, v -> ModernConfig.showWatermark = v),
        });
    }

    public static GuiSubmodOptionsScreen pvpTrackers(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "Waypoints y trackers", new Row[] {
            new Row("Waypoints HUD / 3D", () -> ModernConfig.showWaypoints, v -> ModernConfig.showWaypoints = v),
            new Row("Item tracker 2D", () -> ModernConfig.itemTracker2d, v -> ModernConfig.itemTracker2d = v),
            new Row("Item tracker 3D", () -> ModernConfig.itemTracker3d, v -> ModernConfig.itemTracker3d = v),
        });
    }

    public static GuiSubmodOptionsScreen fps(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "FPS", new Row[] {
            new Row("Mostrar FPS", () -> ModernConfig.showFps, v -> ModernConfig.showFps = v),
            new Row("Bajar FPS sin foco", () -> PerformanceConfig.reduceFpsWhenMinimized, v -> PerformanceConfig.reduceFpsWhenMinimized = v),
        });
    }

    public static GuiSubmodOptionsScreen entity(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "Entity / cull", new Row[] {
            new Row("Entity cull", () -> ModernConfig.entityCull, v -> ModernConfig.entityCull = v),
            new Row("Nametag cull", () -> ModernConfig.nametagCull, v -> ModernConfig.nametagCull = v),
            new Row("Nametag LOD", () -> ModernConfig.nametagLod, v -> ModernConfig.nametagLod = v),
            new Row("Block entity cull", () -> ModernConfig.blockEntityCull, v -> ModernConfig.blockEntityCull = v),
            new Row("Anim freeze lejos", () -> ModernConfig.entityAnimCull, v -> ModernConfig.entityAnimCull = v),
            new Row("Armor stand cull", () -> ModernConfig.armorStandCull, v -> ModernConfig.armorStandCull = v),
            new Row("Item frame cull", () -> ModernConfig.itemFrameCull, v -> ModernConfig.itemFrameCull = v),
        });
    }

    public static GuiSubmodOptionsScreen bedwars(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "BedWars", new Row[] {
            new Row("HUD recursos", () -> ModernConfig.showBedwarsResources, v -> ModernConfig.showBedwarsResources = v),
            new Row("Mostrar nombres", () -> ModernConfig.showItemNames, v -> ModernConfig.showItemNames = v),
            new Row("Fondo transparente", () -> ModernConfig.bwResTransparentBg, v -> ModernConfig.bwResTransparentBg = v),
            new Row("Camas coloridas", () -> ModernConfig.coloredBeds, v -> ModernConfig.coloredBeds = v),
            new Row("Timer bridge", () -> ModernConfig.showBridgeTimer, v -> ModernConfig.showBridgeTimer = v),
            new Row("Perfiles auto por modo", () -> ModernConfig.autoGameModeProfiles, v -> ModernConfig.autoGameModeProfiles = v),
        });
    }

    public static GuiSubmodOptionsScreen chat(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "Chat", new Row[] {
            new Row("Chat triggers", () -> ModernConfig.chatTriggers, v -> ModernConfig.chatTriggers = v),
            new Row("Chat alerts", () -> ModernConfig.chatAlertsEnabled, v -> {
                ModernConfig.chatAlertsEnabled = v;
                com.paraguacraft.pvp.modern.core.ChatAlerts.enabled = v;
                com.paraguacraft.pvp.modern.core.ChatAlerts.save();
            }),
            new Row("Historial ilimitado", () -> ModernConfig.chatUnlimited, v -> ModernConfig.chatUnlimited = v),
            new Row("Sombra de texto", () -> ModernConfig.chatTextShadow, v -> ModernConfig.chatTextShadow = v),
            new Row("Resaltar tu nombre", () -> ModernConfig.chatHighlightName, v -> ModernConfig.chatHighlightName = v),
        });
    }

    public static GuiSubmodOptionsScreen scoreboard(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "Scoreboard", new Row[] {
            new Row("Mostrar scoreboard", () -> ModernConfig.scoreboardEnabled, v -> ModernConfig.scoreboardEnabled = v),
            new Row("Fondo transparente", () -> ModernConfig.scoreboardTransparentBg, v -> ModernConfig.scoreboardTransparentBg = v),
            new Row("Ocultar numeros rojos", () -> ModernConfig.scoreboardHideRedNumbers, v -> ModernConfig.scoreboardHideRedNumbers = v),
            new Row("Ocultar stats", () -> ModernConfig.scoreboardHideStats, v -> ModernConfig.scoreboardHideStats = v),
            new Row("Sombra de texto", () -> ModernConfig.scoreboardTextShadow, v -> ModernConfig.scoreboardTextShadow = v),
            new Row("Escala", () -> true, v -> ModernConfig.cycleScoreboardScale(), () -> ModernConfig.scoreboardScale + "%"),
        });
    }

    public static GuiSubmodOptionsScreen tnt(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "Cuenta TNT", new Row[] {
            new Row("Mostrar countdown", () -> ModernConfig.showTntCountdown, v -> ModernConfig.showTntCountdown = v),
            new Row("Compensar ping", () -> ModernConfig.tntPingCompensate, v -> ModernConfig.tntPingCompensate = v),
            new Row("Fuse", () -> true, v -> ModernConfig.cycleTntFuse(), ModernConfig::tntFuseLabel),
        });
    }

    public static GuiSubmodOptionsScreen tab(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "Editor de Tab", new Row[] {
            new Row("Editor activo", () -> ModernConfig.tabEditor, v -> ModernConfig.tabEditor = v),
            new Row("Ping como numero", () -> ModernConfig.tabPingNumbers, v -> ModernConfig.tabPingNumbers = v),
            new Row("Ocultar NPCs", () -> ModernConfig.tabHideNpcs, v -> ModernConfig.tabHideNpcs = v),
            new Row("Ocultar ping > 500", () -> ModernConfig.tabHideHighPing, v -> ModernConfig.tabHideHighPing = v),
        });
    }

    public static GuiSubmodOptionsScreen hitColor(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "Color de golpe", new Row[] {
            new Row("Color de golpe activo", () -> ModernConfig.hitColorEnabled, v -> ModernConfig.hitColorEnabled = v),
            new Row("Preset", () -> true, v -> ModernConfig.cycleHitColor(), ModernConfig::hitColorLabel),
        });
    }

    public static GuiSubmodOptionsScreen particles(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "Particulas", new Row[] {
            new Row("Modo", () -> true, v -> {
                PerformanceConfig.cycleParticleMode();
                com.paraguacraft.pvp.modern.core.PerformanceBootstrap.applyParticleModeNow(net.minecraft.client.MinecraftClient.getInstance());
            }, PerformanceConfig::particleModeLabel),
            new Row("Ocultar explosiones", () -> PerformanceConfig.hideExplosionParticles, v -> PerformanceConfig.hideExplosionParticles = v),
            new Row("Ocultar particulas de pocion", () -> PerformanceConfig.hidePotionParticles, v -> PerformanceConfig.hidePotionParticles = v),
        });
    }

    public static GuiSubmodOptionsScreen reach(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "Reach display", new Row[] {
            new Row("Mostrar alcance", () -> ModernConfig.reachDisplay, v -> ModernConfig.reachDisplay = v),
            new Row("Mantener ultimo golpe (4s)", () -> ModernConfig.reachPersist, v -> ModernConfig.reachPersist = v),
        });
    }

    public static GuiSubmodOptionsScreen guiScale(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "Escala GUI", new Row[] {
            new Row("Hotbar", () -> true, v -> ModernConfig.cycleHotbarScale(), () -> ModernConfig.hotbarScale + "%"),
            new Row("Inventario", () -> true, v -> ModernConfig.cycleInventoryScale(), () -> ModernConfig.inventoryScale + "%"),
        });
    }

    public static GuiSubmodOptionsScreen packHud(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "Visualizacion de pack", new Row[] {
            new Row("Mostrar pack actual en HUD", () -> ModernConfig.showPackHud, v -> ModernConfig.showPackHud = v),
        });
    }

    public static GuiSubmodOptionsScreen heightLimit(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "Limite de altura", new Row[] {
            new Row("Mostrar altura / techo", () -> ModernConfig.showHeightLimit, v -> ModernConfig.showHeightLimit = v),
            new Row("Techo", () -> true, v -> ModernConfig.cycleHeightLimit(), ModernConfig::heightLimitLabel),
        });
    }
}
