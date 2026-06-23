package com.paraguacraft.pvp.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.client.settings.KeyBinding;
import net.minecraft.util.ChatComponentText;
import net.minecraftforge.fml.client.registry.ClientRegistry;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import net.minecraftforge.fml.common.gameevent.InputEvent.KeyInputEvent;
import net.minecraftforge.fml.common.gameevent.TickEvent.ClientTickEvent;
import net.minecraftforge.event.entity.living.LivingEvent.LivingUpdateEvent;
import org.lwjgl.input.Keyboard;

// ---> CAMBIO: Importamos tu nuevo menú moderno en vez del viejo
import com.paraguacraft.pvp.gui.GuiParaguaMenu; 
import com.paraguacraft.pvp.gui.GuiEditHUD;

public class QoLManager {

    private final Minecraft mc = Minecraft.getMinecraft();
    
    public static boolean toggleSprintActive = true; 
    public static boolean fullbrightActive = true; 

    private KeyBinding toggleSprintKey;
    private KeyBinding fullbrightKey;
    private KeyBinding menuKey;
    public static KeyBinding editHudKey;

    public QoLManager() {
        toggleSprintKey = new KeyBinding("Toggle Sprint", Keyboard.KEY_V, "Paraguacraft PvP");
        fullbrightKey = new KeyBinding("Fullbright", Keyboard.KEY_G, "Paraguacraft PvP");
        menuKey = new KeyBinding("Mod Menu", Keyboard.KEY_RSHIFT, "Paraguacraft PvP");
        editHudKey = new KeyBinding("Editar HUD", Keyboard.KEY_RCONTROL, "Paraguacraft PvP"); 
        
        ClientRegistry.registerKeyBinding(toggleSprintKey);
        ClientRegistry.registerKeyBinding(fullbrightKey);
        ClientRegistry.registerKeyBinding(menuKey);
        ClientRegistry.registerKeyBinding(editHudKey);
    }

    @SubscribeEvent
    public void onKeyInput(KeyInputEvent event) {
        if (toggleSprintKey.isPressed()) {
            toggleSprintActive = !toggleSprintActive;
            sendMessage("Toggle Sprint: " + (toggleSprintActive ? "\u00A7aON" : "\u00A7cOFF"));
        }
        
        if (fullbrightKey.isPressed()) {
            fullbrightActive = !fullbrightActive;
            if (!fullbrightActive) {
                mc.gameSettings.gammaSetting = 1.0F; 
            }
            sendMessage("Fullbright: " + (fullbrightActive ? "\u00A7aON" : "\u00A7cOFF"));
        }

        if (menuKey.isPressed()) {
            // ---> CAMBIO: Abrimos el panel estilo Lunar Client
            mc.displayGuiScreen(new GuiParaguaMenu());
        }

        if (editHudKey.isPressed()) {
            mc.displayGuiScreen(new GuiEditHUD());
        }
    }

    @SubscribeEvent
    public void onClientTick(ClientTickEvent event) {
        if (mc.thePlayer == null) return;

        if (toggleSprintActive && mc.currentScreen == null) {
            KeyBinding.setKeyBindState(mc.gameSettings.keyBindSprint.getKeyCode(), true);
        }

        if (fullbrightActive && mc.gameSettings.gammaSetting < 100.0F) {
            mc.gameSettings.gammaSetting = 100.0F;
        }
    }

    @SubscribeEvent
    public void onPlayerUpdate(LivingUpdateEvent event) {
        if (ModConfig.noHurtCam && event.entityLiving == mc.thePlayer) {
            event.entityLiving.hurtTime = 0;
            event.entityLiving.maxHurtTime = 0;
            event.entityLiving.attackedAtYaw = 0;
        }
    }

    private void sendMessage(String msg) {
        if (mc.thePlayer != null) {
            mc.thePlayer.addChatMessage(new ChatComponentText("\u00A78[\u00A79Paraguacraft\u00A78] \u00A77" + msg));
        }
    }
}