package com.paraguacraft.pvp.core;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.util.Properties;

/** Recuerda el ultimo modo de Hypixel Quick Play para reconectar rapido. */
public final class QuickPlayState {

    public static String lastCommand = "";
    public static String lastLabel = "";
    public static String lastGame = "";

    private QuickPlayState() {}

    public static boolean hasLast() {
        return lastCommand != null && !lastCommand.isEmpty();
    }

    public static void remember(String command, String label, String game) {
        if (command == null || command.isEmpty()) {
            return;
        }
        lastCommand = command;
        lastLabel = label != null ? label : command;
        lastGame = game != null ? game : "";
        persist();
    }

    public static void reconnect() {
        if (!hasLast()) {
            return;
        }
        HypixelHelper.sendCommand(lastCommand);
    }

    public static void load() {
        try {
            File file = propsFile();
            if (!file.isFile()) {
                return;
            }
            Properties props = new Properties();
            props.load(new FileInputStream(file));
            lastCommand = props.getProperty("quickPlayLastCommand", "");
            lastLabel = props.getProperty("quickPlayLastLabel", "");
            lastGame = props.getProperty("quickPlayLastGame", "");
            ModConfig.quickPlayLastCommand = lastCommand;
            ModConfig.quickPlayLastLabel = lastLabel;
            ModConfig.quickPlayLastGame = lastGame;
        } catch (Exception ignored) {}
    }

    private static void persist() {
        try {
            File file = propsFile();
            Properties props = new Properties();
            if (file.isFile()) {
                props.load(new FileInputStream(file));
            }
            props.setProperty("quickPlayLastCommand", lastCommand);
            props.setProperty("quickPlayLastLabel", lastLabel);
            props.setProperty("quickPlayLastGame", lastGame);
            if (file.getParentFile() != null) {
                file.getParentFile().mkdirs();
            }
            props.store(new FileOutputStream(file), "Paraguacraft Quick Play");
        } catch (Exception ignored) {}
        ModConfig.quickPlayLastCommand = lastCommand;
        ModConfig.quickPlayLastLabel = lastLabel;
        ModConfig.quickPlayLastGame = lastGame;
    }

    private static File propsFile() {
        return new File(Minecraft.getMinecraft().mcDataDir, "paraguacraft_v2.properties");
    }
}
