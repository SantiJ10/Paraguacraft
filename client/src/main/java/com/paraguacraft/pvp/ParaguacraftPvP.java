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
import com.paraguacraft.pvp.modules.QoLManager;
import com.paraguacraft.pvp.animations.CustomItemRenderer;
import net.minecraftforge.client.event.GuiOpenEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import net.minecraft.client.gui.GuiMainMenu;
import net.minecraft.client.Minecraft;
import java.lang.reflect.Field;

@Mod(modid = ParaguacraftPvP.MODID, name = ParaguacraftPvP.NAME, version = ParaguacraftPvP.VERSION, acceptedMinecraftVersions = "[1.8.9]")
public class ParaguacraftPvP {

    public static final String MODID = "paraguacraftpvp";
    public static final String NAME = "Paraguacraft PvP Client";
    public static final String VERSION = "1.0.0";

    @Mod.Instance(MODID)
    public static ParaguacraftPvP instance;

    @EventHandler
    public void preInit(FMLPreInitializationEvent event) {
    }

    @EventHandler
    public void init(FMLInitializationEvent event) {
        // Registramos todos los modulos en el Bus de Eventos
        MinecraftForge.EVENT_BUS.register(this);
        MinecraftForge.EVENT_BUS.register(new HUDOverlay());
        MinecraftForge.EVENT_BUS.register(new QoLManager());
        MinecraftForge.EVENT_BUS.register(new GuiBackgroundHandler());
        MinecraftForge.EVENT_BUS.register(new com.paraguacraft.pvp.modules.VisualsManager());

        // Inyeccion de las animaciones 1.7
        try {
            Minecraft mc = Minecraft.getMinecraft();
            Field renderField = null;
            try {
                renderField = mc.entityRenderer.getClass().getDeclaredField("itemRenderer");
            } catch (NoSuchFieldException e) {
                renderField = mc.entityRenderer.getClass().getDeclaredField("field_71415_G");
            }
            if (renderField != null) {
                renderField.setAccessible(true);
                renderField.set(mc.entityRenderer, new CustomItemRenderer(mc));
                System.out.println("[Paraguacraft] Animaciones 1.7 inyectadas correctamente!");
            }
        } catch (Exception e) {
            e.printStackTrace();
            System.out.println("[Paraguacraft] Error al inyectar animaciones 1.7");
        }
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