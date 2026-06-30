package com.paraguacraft.pvp;

import net.minecraftforge.common.MinecraftForge;
import net.minecraftforge.fml.common.Mod;
import net.minecraftforge.fml.common.Mod.EventHandler;
import net.minecraftforge.fml.common.event.FMLInitializationEvent;
import net.minecraftforge.fml.common.event.FMLPostInitializationEvent;
import net.minecraftforge.fml.common.event.FMLPreInitializationEvent;
import com.paraguacraft.pvp.gui.CustomMainMenu;
import com.paraguacraft.pvp.gui.GuiBackgroundHandler;
import com.paraguacraft.pvp.hud.HUDOverlay;
import com.paraguacraft.pvp.core.OptifinePreset;
import com.paraguacraft.pvp.core.PerformanceConfig;
import com.paraguacraft.pvp.core.ModConfigApply;
import com.paraguacraft.pvp.core.LauncherIpcHandler;
import com.paraguacraft.pvp.core.DiscordPresenceHandler;
import com.paraguacraft.pvp.core.HardwarePreset;
import com.paraguacraft.pvp.core.HytilsDefaults;
import com.paraguacraft.pvp.modules.ModConfig;
import com.paraguacraft.pvp.modules.QoLManager;
import com.paraguacraft.pvp.network.BadgeNetHandler;
import net.minecraftforge.client.event.GuiOpenEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import net.minecraft.client.gui.GuiMainMenu;

@Mod(
    modid = ParaguacraftPvP.MODID,
    name = ParaguacraftPvP.NAME,
    version = ParaguacraftPvP.VERSION,
    acceptedMinecraftVersions = "[1.8.9]",
    dependencies = "before:hytils-reborn"
)
public class ParaguacraftPvP {

    public static final String MODID = "paraguacraftpvp";
    public static final String NAME = "Paraguacraft PvP Client";
    public static final String VERSION = "2.1.26";

    @Mod.Instance(MODID)
    public static ParaguacraftPvP instance;

    @EventHandler
    public void preInit(FMLPreInitializationEvent event) {
        HytilsDefaults.applyIfNeeded();
    }

    @EventHandler
    public void init(FMLInitializationEvent event) {
        ModConfig.load();
        ModConfigApply.onStartup();
        HardwarePreset.applyIfEnabled();
        PerformanceConfig.applyParticleLimitsFromMode();
        OptifinePreset.applyIfEnabled();
        MinecraftForge.EVENT_BUS.register(this);
        MinecraftForge.EVENT_BUS.register(new HUDOverlay());
        MinecraftForge.EVENT_BUS.register(new QoLManager());
        MinecraftForge.EVENT_BUS.register(new GuiBackgroundHandler());
        MinecraftForge.EVENT_BUS.register(new com.paraguacraft.pvp.modules.VisualsManager());
        MinecraftForge.EVENT_BUS.register(new BadgeNetHandler());
        MinecraftForge.EVENT_BUS.register(new DiscordPresenceHandler());
        MinecraftForge.EVENT_BUS.register(new LauncherIpcHandler());
        MinecraftForge.EVENT_BUS.register(new com.paraguacraft.pvp.modules.CombatStats());
        MinecraftForge.EVENT_BUS.register(new com.paraguacraft.pvp.modules.ChatTriggerManager());
        net.minecraftforge.client.ClientCommandHandler.instance.registerCommand(new com.paraguacraft.pvp.command.CommandChatAlerts());
        System.out.println("[Paraguacraft V2] Fase C/D — perf, perfiles, keybinds, pantallas carga");
    }

    @EventHandler
    public void postInit(FMLPostInitializationEvent event) {
    }

    @SubscribeEvent
    public void onGuiOpen(GuiOpenEvent event) {
        if (event.gui instanceof GuiMainMenu) {
            event.gui = new CustomMainMenu();
        } else if (event.gui instanceof net.minecraft.client.gui.GuiIngameMenu) {
            // Interceptamos el menu de pausa Vanilla y ponemos el nuestro oscuro
            event.gui = new com.paraguacraft.pvp.gui.CustomPauseMenu();
        }
    }
}