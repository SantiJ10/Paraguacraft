package com.paraguacraft.pvp.core;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.nio.charset.StandardCharsets;

/** Export / import de perfiles de mods (Fase D). */
public final class ModProfileManager {

    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();

    private ModProfileManager() {}

    public static File profilesDir() {
        File dir = new File(Minecraft.getMinecraft().mcDataDir, "paraguacraft/profiles");
        if (!dir.exists()) {
            dir.mkdirs();
        }
        return dir;
    }

    public static void exportTo(File dest) throws Exception {
        JsonObject root = new JsonObject();
        root.addProperty("version", 1);
        root.addProperty("name", stripExtension(dest.getName()));

        JsonObject mods = new JsonObject();
        mods.addProperty("showFPS", ModConfig.showFPS);
        mods.addProperty("showPing", ModConfig.showPing);
        mods.addProperty("showCPS", ModConfig.showCPS);
        mods.addProperty("showKeystrokes", ModConfig.showKeystrokes);
        mods.addProperty("noHurtCam", ModConfig.noHurtCam);
        mods.addProperty("showArmor", ModConfig.showArmor);
        mods.addProperty("showPotions", ModConfig.showPotions);
        mods.addProperty("scoreboardEnabled", ModConfig.scoreboardEnabled);
        mods.addProperty("scoreboardTransparentBg", ModConfig.scoreboardTransparentBg);
        mods.addProperty("scoreboardHideRedNumbers", ModConfig.scoreboardHideRedNumbers);
        mods.addProperty("scoreboardHideStats", ModConfig.scoreboardHideStats);
        mods.addProperty("transparentScoreboard", ModConfig.transparentScoreboard);
        mods.addProperty("dynamicFov", ModConfig.dynamicFov);
        mods.addProperty("crosshairMode", ModConfig.crosshairMode);
        mods.addProperty("showCoords", ModConfig.showCoords);
        mods.addProperty("toggleSneak", ModConfig.toggleSneak);
        mods.addProperty("showArmorPercentage", ModConfig.showArmorPercentage);
        mods.addProperty("windowedFullscreen", ModConfig.windowedFullscreen);
        mods.addProperty("toggleSprintActive", ModConfig.toggleSprintActive);
        mods.addProperty("toggleSprintLegacyActive", ModConfig.toggleSprintLegacyActive);
        mods.addProperty("fullbrightActive", ModConfig.fullbrightActive);
        mods.addProperty("showHeldItem", ModConfig.showHeldItem);
        mods.addProperty("showServerHUD", ModConfig.showServerHUD);
        mods.addProperty("showCompass", ModConfig.showCompass);
        mods.addProperty("showNametagLogo", ModConfig.showNametagLogo);
        mods.addProperty("showNametagLogoOthers", ModConfig.showNametagLogoOthers);
        mods.addProperty("keyMenu", ModConfig.keyMenu);
        mods.addProperty("keyEditHud", ModConfig.keyEditHud);
        mods.addProperty("keyToggleSprint", ModConfig.keyToggleSprint);
        mods.addProperty("keyToggleSprintLegacy", ModConfig.keyToggleSprintLegacy);
        mods.addProperty("keyFullbright", ModConfig.keyFullbright);
        root.add("mods", mods);

        JsonObject perf = new JsonObject();
        perf.addProperty("boostFps", PerformanceConfig.boostFps);
        perf.addProperty("oldAnimations", PerformanceConfig.oldAnimations);
        perf.addProperty("entityCull", PerformanceConfig.entityCull);
        perf.addProperty("nametagCull", PerformanceConfig.nametagCull);
        perf.addProperty("entityAnimCull", PerformanceConfig.entityAnimCull);
        perf.addProperty("blockEntityCull", PerformanceConfig.blockEntityCull);
        perf.addProperty("particleMode", PerformanceConfig.particleMode.name());
        perf.addProperty("memoryCleanup", PerformanceConfig.memoryCleanupOnWorldChange);
        perf.addProperty("applyVanillaPreset", PerformanceConfig.applyVanillaPreset);
        perf.addProperty("armorStandCull", PerformanceConfig.armorStandCull);
        perf.addProperty("itemFrameCull", PerformanceConfig.itemFrameCull);
        perf.addProperty("nametagLod", PerformanceConfig.nametagLod);
        perf.addProperty("skipCombatFx", PerformanceConfig.skipCombatFx);
        perf.addProperty("hardwareAutoPreset", PerformanceConfig.hardwareAutoPreset);
        root.add("performance", perf);

        try (OutputStreamWriter w = new OutputStreamWriter(new FileOutputStream(dest), StandardCharsets.UTF_8)) {
            w.write(GSON.toJson(root));
        }
    }

    public static void importFrom(File src) throws Exception {
        JsonObject root;
        try (InputStreamReader r = new InputStreamReader(new FileInputStream(src), StandardCharsets.UTF_8)) {
            root = new JsonParser().parse(r).getAsJsonObject();
        }
        if (root.has("mods")) {
            JsonObject m = root.getAsJsonObject("mods");
            ModConfig.showFPS = bool(m, "showFPS", ModConfig.showFPS);
            ModConfig.showPing = bool(m, "showPing", ModConfig.showPing);
            ModConfig.showCPS = bool(m, "showCPS", ModConfig.showCPS);
            ModConfig.showKeystrokes = bool(m, "showKeystrokes", ModConfig.showKeystrokes);
            ModConfig.noHurtCam = bool(m, "noHurtCam", ModConfig.noHurtCam);
            ModConfig.showArmor = bool(m, "showArmor", ModConfig.showArmor);
            ModConfig.showPotions = bool(m, "showPotions", ModConfig.showPotions);
            ModConfig.scoreboardEnabled = bool(m, "scoreboardEnabled", ModConfig.scoreboardEnabled);
            ModConfig.scoreboardTransparentBg = bool(m, "scoreboardTransparentBg", ModConfig.scoreboardTransparentBg);
            ModConfig.scoreboardHideRedNumbers = bool(m, "scoreboardHideRedNumbers", ModConfig.scoreboardHideRedNumbers);
            ModConfig.scoreboardHideStats = bool(m, "scoreboardHideStats", ModConfig.scoreboardHideStats);
            ModConfig.transparentScoreboard = bool(m, "transparentScoreboard", ModConfig.transparentScoreboard);
            ModConfig.dynamicFov = bool(m, "dynamicFov", ModConfig.dynamicFov);
            ModConfig.crosshairMode = num(m, "crosshairMode", ModConfig.crosshairMode);
            ModConfig.showCoords = bool(m, "showCoords", ModConfig.showCoords);
            ModConfig.toggleSneak = bool(m, "toggleSneak", ModConfig.toggleSneak);
            ModConfig.showArmorPercentage = bool(m, "showArmorPercentage", ModConfig.showArmorPercentage);
            ModConfig.windowedFullscreen = bool(m, "windowedFullscreen", ModConfig.windowedFullscreen);
            ModConfig.toggleSprintActive = bool(m, "toggleSprintActive", ModConfig.toggleSprintActive);
            ModConfig.toggleSprintLegacyActive = bool(m, "toggleSprintLegacyActive", ModConfig.toggleSprintLegacyActive);
            ModConfig.fullbrightActive = bool(m, "fullbrightActive", ModConfig.fullbrightActive);
            ModConfig.showHeldItem = bool(m, "showHeldItem", ModConfig.showHeldItem);
            ModConfig.showServerHUD = bool(m, "showServerHUD", ModConfig.showServerHUD);
            ModConfig.showCompass = bool(m, "showCompass", ModConfig.showCompass);
            ModConfig.showNametagLogo = bool(m, "showNametagLogo", ModConfig.showNametagLogo);
            ModConfig.showNametagLogoOthers = bool(m, "showNametagLogoOthers", ModConfig.showNametagLogoOthers);
            ModConfig.keyMenu = num(m, "keyMenu", ModConfig.keyMenu);
            ModConfig.keyEditHud = num(m, "keyEditHud", ModConfig.keyEditHud);
            ModConfig.keyToggleSprint = num(m, "keyToggleSprint", ModConfig.keyToggleSprint);
            ModConfig.keyToggleSprintLegacy = num(m, "keyToggleSprintLegacy", ModConfig.keyToggleSprintLegacy);
            ModConfig.keyFullbright = num(m, "keyFullbright", ModConfig.keyFullbright);
        }
        if (root.has("performance")) {
            JsonObject p = root.getAsJsonObject("performance");
            PerformanceConfig.boostFps = bool(p, "boostFps", PerformanceConfig.boostFps);
            PerformanceConfig.oldAnimations = bool(p, "oldAnimations", PerformanceConfig.oldAnimations);
            PerformanceConfig.entityCull = bool(p, "entityCull", PerformanceConfig.entityCull);
            PerformanceConfig.nametagCull = bool(p, "nametagCull", PerformanceConfig.nametagCull);
            PerformanceConfig.entityAnimCull = bool(p, "entityAnimCull", PerformanceConfig.entityAnimCull);
            PerformanceConfig.blockEntityCull = bool(p, "blockEntityCull", PerformanceConfig.blockEntityCull);
            PerformanceConfig.particleMode = PerformanceConfig.ParticleMode.fromName(
                str(p, "particleMode", PerformanceConfig.particleMode.name())
            );
            PerformanceConfig.memoryCleanupOnWorldChange = bool(p, "memoryCleanup", PerformanceConfig.memoryCleanupOnWorldChange);
            PerformanceConfig.applyVanillaPreset = bool(p, "applyVanillaPreset", PerformanceConfig.applyVanillaPreset);
            PerformanceConfig.armorStandCull = bool(p, "armorStandCull", PerformanceConfig.armorStandCull);
            PerformanceConfig.itemFrameCull = bool(p, "itemFrameCull", PerformanceConfig.itemFrameCull);
            PerformanceConfig.nametagLod = bool(p, "nametagLod", PerformanceConfig.nametagLod);
            PerformanceConfig.skipCombatFx = bool(p, "skipCombatFx", PerformanceConfig.skipCombatFx);
            PerformanceConfig.hardwareAutoPreset = bool(p, "hardwareAutoPreset", PerformanceConfig.hardwareAutoPreset);
            PerformanceConfig.applyParticleLimitsFromMode();
        }
        ModConfig.save();
        ModConfigApply.onStartup();
        QoLKeybinds.applyFromConfig();
    }

    private static boolean bool(JsonObject o, String key, boolean def) {
        return o.has(key) ? o.get(key).getAsBoolean() : def;
    }

    private static int num(JsonObject o, String key, int def) {
        return o.has(key) ? o.get(key).getAsInt() : def;
    }

    private static String str(JsonObject o, String key, String def) {
        return o.has(key) ? o.get(key).getAsString() : def;
    }

    private static String stripExtension(String name) {
        int dot = name.lastIndexOf('.');
        return dot > 0 ? name.substring(0, dot) : name;
    }
}
