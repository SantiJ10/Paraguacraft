package com.paraguacraft.pvp.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.client.entity.EntityPlayerSP;
import net.minecraft.client.settings.KeyBinding;
import net.minecraft.util.ChatComponentText;
import net.minecraftforge.fml.client.registry.ClientRegistry;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import net.minecraftforge.fml.common.gameevent.InputEvent.KeyInputEvent;
import net.minecraftforge.fml.common.gameevent.TickEvent.ClientTickEvent;
import net.minecraftforge.event.entity.living.LivingEvent.LivingUpdateEvent;
import org.lwjgl.input.Keyboard;

import com.paraguacraft.pvp.gui.GuiParaguaMenu;
import com.paraguacraft.pvp.gui.GuiEditHUD;
import com.paraguacraft.pvp.gui.GuiHypixelQuickPlay;
import com.paraguacraft.pvp.modules.FreelookManager;

public class QoLManager {

    private final Minecraft mc = Minecraft.getMinecraft();

    public static KeyBinding toggleSprintKey;
    public static KeyBinding toggleSprintLegacyKey;
    public static KeyBinding fullbrightKey;
    public static KeyBinding menuKey;
    public static KeyBinding editHudKey;
    public static KeyBinding freelookKey;
    public static KeyBinding quickPlayKey;

    private boolean sneakKeyWasPressed = false;

    public QoLManager() {
        toggleSprintKey = new KeyBinding("Toggle Sprint (M)", ModConfig.keyToggleSprint, "Paraguacraft PvP");
        toggleSprintLegacyKey = new KeyBinding("Toggle Sprint legacy (N)", ModConfig.keyToggleSprintLegacy, "Paraguacraft PvP");
        fullbrightKey = new KeyBinding("Fullbright", ModConfig.keyFullbright, "Paraguacraft PvP");
        menuKey = new KeyBinding("Mod Menu", ModConfig.keyMenu, "Paraguacraft PvP");
        editHudKey = new KeyBinding("Editar HUD", ModConfig.keyEditHud, "Paraguacraft PvP");
        freelookKey = new KeyBinding("Freelook", ModConfig.keyFreelook, "Paraguacraft PvP");
        quickPlayKey = new KeyBinding("Hypixel Quick Play", ModConfig.keyQuickPlay, "Paraguacraft PvP");

        ClientRegistry.registerKeyBinding(toggleSprintKey);
        ClientRegistry.registerKeyBinding(toggleSprintLegacyKey);
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
            sendMessage("Correr toggle (M): " + (ModConfig.toggleSprintActive ? "\u00A7aON" : "\u00A7cOFF"));
        }

        if (toggleSprintLegacyKey.isPressed()) {
            ModConfig.toggleSprintLegacyActive = !ModConfig.toggleSprintLegacyActive;
            ModConfig.save();
            sendMessage("Correr toggle legacy (N): " + (ModConfig.toggleSprintLegacyActive ? "\u00A7aON" : "\u00A7cOFF"));
        }

        if (ModConfig.toggleSneak && mc.thePlayer != null) {
            int sneakKey = mc.gameSettings.keyBindSneak.getKeyCode();
            boolean sneakDown = Keyboard.isKeyDown(sneakKey);
            if (sneakDown && !sneakKeyWasPressed) {
                ModConfig.isSneakingToggled = !ModConfig.isSneakingToggled;
                EntityPlayerSP player = mc.thePlayer;
                player.setSneaking(ModConfig.isSneakingToggled);
                player.movementInput.sneak = ModConfig.isSneakingToggled;
            }
            sneakKeyWasPressed = sneakDown;
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
        if (mc.thePlayer == null) {
            return;
        }

        if (ModConfig.freelookEnabled) {
            if (freelookKey.isKeyDown()) {
                if (!FreelookManager.active) {
                    FreelookManager.onPress();
                }
            } else if (FreelookManager.active) {
                FreelookManager.onRelease();
            }
        }

        if (ModConfig.fullbrightActive && mc.gameSettings.gammaSetting < 100.0F) {
            mc.gameSettings.gammaSetting = 100.0F;
        }

        if (!ModConfig.toggleSneak && ModConfig.isSneakingToggled) {
            ModConfig.isSneakingToggled = false;
            sneakKeyWasPressed = false;
        }
    }

    @SubscribeEvent
    public void onPlayerUpdate(LivingUpdateEvent event) {
        if (!event.entityLiving.worldObj.isRemote) {
            return;
        }

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
