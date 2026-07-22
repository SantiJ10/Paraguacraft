package com.paraguacraft.pvp.modern.core;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.fabricmc.loader.api.FabricLoader;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/** Export / import de perfiles de mods (paridad con `ModProfileManager` 1.8.9). */
public final class ModProfileManager {

    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();

    private ModProfileManager() {}

    public static File profilesDir() {
        File dir = FabricLoader.getInstance().getGameDir().resolve("paraguacraft/profiles").toFile();
        if (!dir.exists()) {
            dir.mkdirs();
        }
        return dir;
    }

    public static List<File> listProfiles() {
        File[] files = profilesDir().listFiles((d, name) -> name.endsWith(".json"));
        List<File> out = new ArrayList<>();
        if (files != null) {
            out.addAll(List.of(files));
        }
        out.sort(Comparator.comparing(File::getName));
        return out;
    }

    public static void exportTo(File dest) throws Exception {
        JsonObject root = new JsonObject();
        root.addProperty("version", 1);
        root.addProperty("name", stripExtension(dest.getName()));

        JsonObject mods = new JsonObject();
        mods.addProperty("showFps", ModernConfig.showFps);
        mods.addProperty("showPing", ModernConfig.showPing);
        mods.addProperty("showCps", ModernConfig.showCps);
        mods.addProperty("showKeystrokes", ModernConfig.showKeystrokes);
        mods.addProperty("noHurtCam", ModernConfig.noHurtCam);
        mods.addProperty("showArmor", ModernConfig.showArmor);
        mods.addProperty("showArmorPercentage", ModernConfig.showArmorPercentage);
        mods.addProperty("showPotions", ModernConfig.showPotions);
        mods.addProperty("scoreboardEnabled", ModernConfig.scoreboardEnabled);
        mods.addProperty("scoreboardTransparentBg", ModernConfig.scoreboardTransparentBg);
        mods.addProperty("scoreboardHideRedNumbers", ModernConfig.scoreboardHideRedNumbers);
        mods.addProperty("scoreboardHideStats", ModernConfig.scoreboardHideStats);
        mods.addProperty("dynamicFov", ModernConfig.dynamicFov);
        mods.addProperty("crosshairMode", ModernConfig.crosshairMode);
        mods.addProperty("showCoords", ModernConfig.showCoords);
        mods.addProperty("toggleSneak", ModernConfig.toggleSneak);
        mods.addProperty("toggleSprint", ModernConfig.toggleSprint);
        mods.addProperty("toggleSprintLegacy", ModernConfig.toggleSprintLegacy);
        mods.addProperty("windowedFullscreen", ModernConfig.windowedFullscreen);
        mods.addProperty("fullbright", ModernConfig.fullbright);
        mods.addProperty("showHeldItem", ModernConfig.showHeldItem);
        mods.addProperty("showCompass", ModernConfig.showCompass);
        mods.addProperty("reachDisplay", ModernConfig.reachDisplay);
        mods.addProperty("comboCounter", ModernConfig.comboCounter);
        mods.addProperty("chatTriggers", ModernConfig.chatTriggers);
        mods.addProperty("chatAlertsEnabled", ModernConfig.chatAlertsEnabled);
        mods.addProperty("freelookEnabled", ModernConfig.freelookEnabled);
        mods.addProperty("freelookBlacklistServers", ModernConfig.freelookBlacklistServers);
        mods.addProperty("shaderAutoOffInMatch", ModernConfig.shaderAutoOffInMatch);
        mods.addProperty("reachDisplayPracticeOnly", ModernConfig.reachDisplayPracticeOnly);
        mods.addProperty("autoGameModeProfiles", ModernConfig.autoGameModeProfiles);
        mods.addProperty("showGameModeHud", ModernConfig.showGameModeHud);
        mods.addProperty("showBridgeTimer", ModernConfig.showBridgeTimer);
        mods.addProperty("secondaryResourcePack", ModernConfig.secondaryResourcePack == null ? "" : ModernConfig.secondaryResourcePack);
        mods.addProperty("gameModeHudX", ModernConfig.gameModeHudX);
        mods.addProperty("gameModeHudY", ModernConfig.gameModeHudY);
        mods.addProperty("bridgeTimerX", ModernConfig.bridgeTimerX);
        mods.addProperty("bridgeTimerY", ModernConfig.bridgeTimerY);
        mods.addProperty("reachDisplayX", ModernConfig.reachDisplayX);
        mods.addProperty("reachDisplayY", ModernConfig.reachDisplayY);
        mods.addProperty("showNametagLogo", ModernConfig.showNametagLogo);
        mods.addProperty("showNametagLogoOthers", ModernConfig.showNametagLogoOthers);
        mods.addProperty("showOpponentPing", ModernConfig.showOpponentPing);
        mods.addProperty("showServerHud", ModernConfig.showServerHud);
        mods.addProperty("showMusicHud", ModernConfig.showMusicHud);
        mods.addProperty("showMusicAlbumArt", ModernConfig.showMusicAlbumArt);
        mods.addProperty("showCombatStatsHud", ModernConfig.showCombatStatsHud);
        root.add("mods", mods);

        JsonObject perf = new JsonObject();
        perf.addProperty("boostFps", PerformanceConfig.boostFps);
        perf.addProperty("entityCull", ModernConfig.entityCull);
        perf.addProperty("nametagCull", ModernConfig.nametagCull);
        perf.addProperty("particleMode", PerformanceConfig.particleMode.name());
        perf.addProperty("memoryCleanup", PerformanceConfig.memoryCleanupOnWorldChange);
        perf.addProperty("applyVanillaPreset", PerformanceConfig.applyVanillaPreset);
        perf.addProperty("skipCombatFx", PerformanceConfig.skipCombatFx);
        perf.addProperty("reduceFpsWhenMinimized", PerformanceConfig.reduceFpsWhenMinimized);
        root.add("performance", perf);

        try (OutputStreamWriter w = new OutputStreamWriter(new FileOutputStream(dest), StandardCharsets.UTF_8)) {
            w.write(GSON.toJson(root));
        }
    }

    public static void importFrom(File src) throws Exception {
        JsonObject root;
        try (InputStreamReader r = new InputStreamReader(new FileInputStream(src), StandardCharsets.UTF_8)) {
            root = JsonParser.parseReader(r).getAsJsonObject();
        }
        if (root.has("mods")) {
            JsonObject m = root.getAsJsonObject("mods");
            ModernConfig.showFps = bool(m, "showFps", ModernConfig.showFps);
            ModernConfig.showPing = bool(m, "showPing", ModernConfig.showPing);
            ModernConfig.showCps = bool(m, "showCps", ModernConfig.showCps);
            ModernConfig.showKeystrokes = bool(m, "showKeystrokes", ModernConfig.showKeystrokes);
            ModernConfig.noHurtCam = bool(m, "noHurtCam", ModernConfig.noHurtCam);
            ModernConfig.showArmor = bool(m, "showArmor", ModernConfig.showArmor);
            ModernConfig.showArmorPercentage = bool(m, "showArmorPercentage", ModernConfig.showArmorPercentage);
            ModernConfig.showPotions = bool(m, "showPotions", ModernConfig.showPotions);
            ModernConfig.scoreboardEnabled = bool(m, "scoreboardEnabled", ModernConfig.scoreboardEnabled);
            ModernConfig.scoreboardTransparentBg = bool(m, "scoreboardTransparentBg", ModernConfig.scoreboardTransparentBg);
            ModernConfig.scoreboardHideRedNumbers = bool(m, "scoreboardHideRedNumbers", ModernConfig.scoreboardHideRedNumbers);
            ModernConfig.scoreboardHideStats = bool(m, "scoreboardHideStats", ModernConfig.scoreboardHideStats);
            ModernConfig.dynamicFov = bool(m, "dynamicFov", ModernConfig.dynamicFov);
            ModernConfig.crosshairMode = num(m, "crosshairMode", ModernConfig.crosshairMode);
            ModernConfig.showCoords = bool(m, "showCoords", ModernConfig.showCoords);
            ModernConfig.toggleSneak = bool(m, "toggleSneak", ModernConfig.toggleSneak);
            ModernConfig.toggleSprint = bool(m, "toggleSprint", ModernConfig.toggleSprint);
            ModernConfig.toggleSprintLegacy = bool(m, "toggleSprintLegacy", ModernConfig.toggleSprintLegacy);
            ModernConfig.windowedFullscreen = bool(m, "windowedFullscreen", ModernConfig.windowedFullscreen);
            ModernConfig.fullbright = bool(m, "fullbright", ModernConfig.fullbright);
            ModernConfig.showHeldItem = bool(m, "showHeldItem", ModernConfig.showHeldItem);
            ModernConfig.showCompass = bool(m, "showCompass", ModernConfig.showCompass);
            ModernConfig.reachDisplay = bool(m, "reachDisplay", ModernConfig.reachDisplay);
            ModernConfig.comboCounter = bool(m, "comboCounter", ModernConfig.comboCounter);
            ModernConfig.chatTriggers = bool(m, "chatTriggers", ModernConfig.chatTriggers);
            ModernConfig.chatAlertsEnabled = bool(m, "chatAlertsEnabled", ModernConfig.chatAlertsEnabled);
            ModernConfig.freelookEnabled = bool(m, "freelookEnabled", ModernConfig.freelookEnabled);
            ModernConfig.freelookBlacklistServers = bool(m, "freelookBlacklistServers", ModernConfig.freelookBlacklistServers);
            ModernConfig.shaderAutoOffInMatch = bool(m, "shaderAutoOffInMatch", ModernConfig.shaderAutoOffInMatch);
            ModernConfig.reachDisplayPracticeOnly = bool(m, "reachDisplayPracticeOnly", ModernConfig.reachDisplayPracticeOnly);
            ModernConfig.autoGameModeProfiles = bool(m, "autoGameModeProfiles", ModernConfig.autoGameModeProfiles);
            ModernConfig.showGameModeHud = bool(m, "showGameModeHud", ModernConfig.showGameModeHud);
            ModernConfig.showBridgeTimer = bool(m, "showBridgeTimer", ModernConfig.showBridgeTimer);
            ModernConfig.secondaryResourcePack = str(m, "secondaryResourcePack", ModernConfig.secondaryResourcePack);
            ModernConfig.gameModeHudX = num(m, "gameModeHudX", ModernConfig.gameModeHudX);
            ModernConfig.gameModeHudY = num(m, "gameModeHudY", ModernConfig.gameModeHudY);
            ModernConfig.bridgeTimerX = num(m, "bridgeTimerX", ModernConfig.bridgeTimerX);
            ModernConfig.bridgeTimerY = num(m, "bridgeTimerY", ModernConfig.bridgeTimerY);
            ModernConfig.reachDisplayX = num(m, "reachDisplayX", ModernConfig.reachDisplayX);
            ModernConfig.reachDisplayY = num(m, "reachDisplayY", ModernConfig.reachDisplayY);
            ModernConfig.showNametagLogo = bool(m, "showNametagLogo", ModernConfig.showNametagLogo);
            ModernConfig.showNametagLogoOthers = bool(m, "showNametagLogoOthers", ModernConfig.showNametagLogoOthers);
            ModernConfig.showOpponentPing = bool(m, "showOpponentPing", ModernConfig.showOpponentPing);
            ModernConfig.showServerHud = bool(m, "showServerHud", ModernConfig.showServerHud);
            ModernConfig.showMusicHud = bool(m, "showMusicHud", ModernConfig.showMusicHud);
            ModernConfig.showMusicAlbumArt = bool(m, "showMusicAlbumArt", ModernConfig.showMusicAlbumArt);
            ModernConfig.showCombatStatsHud = bool(m, "showCombatStatsHud", ModernConfig.showCombatStatsHud);
        }
        if (root.has("performance")) {
            JsonObject p = root.getAsJsonObject("performance");
            PerformanceConfig.boostFps = bool(p, "boostFps", PerformanceConfig.boostFps);
            ModernConfig.entityCull = bool(p, "entityCull", ModernConfig.entityCull);
            ModernConfig.nametagCull = bool(p, "nametagCull", ModernConfig.nametagCull);
            PerformanceConfig.particleMode = PerformanceConfig.ParticleMode.fromName(
                str(p, "particleMode", PerformanceConfig.particleMode.name())
            );
            PerformanceConfig.memoryCleanupOnWorldChange = bool(p, "memoryCleanup", PerformanceConfig.memoryCleanupOnWorldChange);
            PerformanceConfig.applyVanillaPreset = bool(p, "applyVanillaPreset", PerformanceConfig.applyVanillaPreset);
            PerformanceConfig.skipCombatFx = bool(p, "skipCombatFx", PerformanceConfig.skipCombatFx);
            PerformanceConfig.reduceFpsWhenMinimized = bool(p, "reduceFpsWhenMinimized", PerformanceConfig.reduceFpsWhenMinimized);
        }
        ModernConfig.save();
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
