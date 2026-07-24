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

/** Submods agrupados (Armadura, FPS, Entity, BedWars, Chat, Scoreboard). */
public class GuiSubmodOptionsScreen extends ParaguacraftScreen {

    public record Row(String label, BooleanSupplier getter, Consumer<Boolean> setter) {}

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
            String value = on ? "ON" : "OFF";
            ctx.drawText(textRenderer, Text.literal(value), px + 288 - textRenderer.getWidth(value), rowY + 7, on ? 0xFF22CC66 : 0xFFCC4444, true);
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
                rows[i].setter.accept(!rows[i].getter.getAsBoolean());
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
            new Row("Fondo transparente", () -> ModernConfig.bwResTransparentBg, v -> ModernConfig.bwResTransparentBg = v),
            new Row("Camas coloridas", () -> ModernConfig.coloredBeds, v -> ModernConfig.coloredBeds = v),
            new Row("Timer bridge", () -> ModernConfig.showBridgeTimer, v -> ModernConfig.showBridgeTimer = v),
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
        });
    }

    public static GuiSubmodOptionsScreen scoreboard(Screen parent) {
        return new GuiSubmodOptionsScreen(parent, "Scoreboard", new Row[] {
            new Row("Mostrar scoreboard", () -> ModernConfig.scoreboardEnabled, v -> ModernConfig.scoreboardEnabled = v),
            new Row("Fondo transparente", () -> ModernConfig.scoreboardTransparentBg, v -> ModernConfig.scoreboardTransparentBg = v),
            new Row("Ocultar numeros rojos", () -> ModernConfig.scoreboardHideRedNumbers, v -> ModernConfig.scoreboardHideRedNumbers = v),
            new Row("Ocultar stats", () -> ModernConfig.scoreboardHideStats, v -> ModernConfig.scoreboardHideStats = v),
        });
    }
}
