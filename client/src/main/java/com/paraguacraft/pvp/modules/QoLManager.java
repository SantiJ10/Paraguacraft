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
import com.paraguacraft.pvp.gui.GuiHypixelQuickPlay;
import com.paraguacraft.pvp.core.BorderlessWindowManager;
import com.paraguacraft.pvp.modules.FreelookManager;

public class QoLManager {

    private final Minecraft mc = Minecraft.getMinecraft();
    
    public static KeyBinding toggleSprintKey;
    public static KeyBinding fullbrightKey;
    public static KeyBinding menuKey;
    public static KeyBinding editHudKey;
    public static KeyBinding freelookKey;
    public static KeyBinding quickPlayKey;

    public QoLManager() {
        toggleSprintKey = new KeyBinding("Toggle Sprint", ModConfig.keyToggleSprint, "Paraguacraft PvP");
        fullbrightKey = new KeyBinding("Fullbright", ModConfig.keyFullbright, "Paraguacraft PvP");
        menuKey = new KeyBinding("Mod Menu", ModConfig.keyMenu, "Paraguacraft PvP");
        editHudKey = new KeyBinding("Editar HUD", ModConfig.keyEditHud, "Paraguacraft PvP");
        freelookKey = new KeyBinding("Freelook", ModConfig.keyFreelook, "Paraguacraft PvP");
        quickPlayKey = new KeyBinding("Hypixel Quick Play", ModConfig.keyQuickPlay, "Paraguacraft PvP");
        
        ClientRegistry.registerKeyBinding(toggleSprintKey);
        ClientRegistry.registerKeyBinding(fullbrightKey);
        ClientRegistry.registerKeyBinding(menuKey);
        ClientRegistry.registerKeyBinding(editHudKey);
        ClientRegistry.registerKeyBinding(freelookKey);
        ClientRegistry.registerKeyBinding(quickPlayKey);
    }

    @SubscribeEvent
    public void onKeyInput(KeyInputEvent event) {
        if (toggleSprintKey.isPressed()) {
            ModConfig.toggleSprintActive = !ModConfig.toggleSprintActive;
            ModConfig.save();
            sendMessage("Toggle Sprint: " + (ModConfig.toggleSprintActive ? "\u00A7aON" : "\u00A7cOFF"));
        }
        
        if (fullbrightKey.isPressed()) {
            ModConfig.fullbrightActive = !ModConfig.fullbrightActive;
            if (!ModConfig.fullbrightActive) {
                mc.gameSettings.gammaSetting = 1.0F; 
            }
            ModConfig.save();
            sendMessage("Fullbright: " + (ModConfig.fullbrightActive ? "\u00A7aON" : "\u00A7cOFF"));
        }

        if (menuKey.isPressed()) {
            // ---> CAMBIO: Abrimos el panel estilo Lunar Client
            mc.displayGuiScreen(new GuiParaguaMenu());
        }

        if (editHudKey.isPressed()) {
            mc.displayGuiScreen(new GuiEditHUD());
        }

        if (quickPlayKey.isPressed()) {
            mc.displayGuiScreen(new GuiHypixelQuickPlay());
        }
    }

    @SubscribeEvent
    public void onClientTick(ClientTickEvent event) {
        if (mc.thePlayer == null) return;

        if (ModConfig.freelookEnabled) {
            if (freelookKey.isKeyDown()) {
                if (!FreelookManager.active) {
                    FreelookManager.onPress();
                }
            } else if (FreelookManager.active) {
                FreelookManager.onRelease();
            }
        }

        if (ModConfig.toggleSprintActive && mc.currentScreen == null) {
            KeyBinding.setKeyBindState(mc.gameSettings.keyBindSprint.getKeyCode(), true);
        }

        if (ModConfig.fullbrightActive && mc.gameSettings.gammaSetting < 100.0F) {
            mc.gameSettings.gammaSetting = 100.0F;
        }

        BorderlessWindowManager.clientTick();
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