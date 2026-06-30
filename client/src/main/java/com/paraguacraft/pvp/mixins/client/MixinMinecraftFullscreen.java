package com.paraguacraft.pvp.mixins.client;

import com.paraguacraft.pvp.modules.ModConfig;
import net.minecraft.client.Minecraft;
import org.lwjgl.input.Mouse;
import org.lwjgl.opengl.Display;
import org.lwjgl.opengl.DisplayMode;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Windowed Fullscreen (pantalla completa en ventana) estilo Patcher/Lunar.
 *
 * Reemplaza la pantalla completa EXCLUSIVA de vanilla (F11) por una ventana SIN
 * bordes del tamaño del escritorio. Permite alt-tab instantáneo, capturar con
 * OBS/Discord y enfocar otras ventanas, sin el parpadeo del fullscreen exclusivo.
 *
 * Técnica (LWJGL2, la misma que usa Patcher): propiedad
 * "org.lwjgl.opengl.Window.undecorated" + Display.setDisplayMode al tamaño del
 * escritorio + Display.setFullscreen(false). El toggle de setResizable fuerza a
 * LWJGL a reaplicar el estilo de la ventana (SWP_FRAMECHANGED por debajo).
 */
@Mixin(Minecraft.class)
public abstract class MixinMinecraftFullscreen {

    @Shadow private boolean fullscreen;
    @Shadow public int displayWidth;
    @Shadow public int displayHeight;
    @Shadow private int tempDisplayWidth;
    @Shadow private int tempDisplayHeight;

    @Inject(method = "toggleFullscreen", at = @At("HEAD"), cancellable = true)
    private void paraguacraft$windowedFullscreen(CallbackInfo ci) {
        // Interceptamos si el modo está activado, o si aún estamos en borderless
        // (para poder salir limpio aunque se haya desactivado la opción).
        if ((!ModConfig.windowedFullscreen && !ModConfig.windowedActive) || !isWindows()) {
            return;
        }
        if (paraguacraft$applyWindowed()) {
            ci.cancel();
        }
    }

    private boolean paraguacraft$applyWindowed() {
        boolean grabbed = Mouse.isGrabbed();
        try {
            this.fullscreen = !this.fullscreen;

            if (grabbed) {
                Mouse.setGrabbed(false);
            }

            DisplayMode mode;
            if (this.fullscreen) {
                // Guardamos el tamaño actual para poder restaurarlo (vanilla lo
                // hacía, pero como cancelamos su método lo hacemos nosotros).
                this.tempDisplayWidth = this.displayWidth > 0 ? this.displayWidth : 854;
                this.tempDisplayHeight = this.displayHeight > 0 ? this.displayHeight : 480;

                System.setProperty("org.lwjgl.opengl.Window.undecorated", "true");
                mode = Display.getDesktopDisplayMode();
                Display.setDisplayMode(mode);
                Display.setLocation(0, 0);
            } else {
                System.setProperty("org.lwjgl.opengl.Window.undecorated", "false");
                mode = new DisplayMode(this.tempDisplayWidth, this.tempDisplayHeight);
                Display.setDisplayMode(mode);
                centerWindow(mode);
            }

            // NUNCA fullscreen exclusivo: así se mantiene el alt-tab y la captura.
            Display.setFullscreen(false);

            this.displayWidth = Math.max(mode.getWidth(), 1);
            this.displayHeight = Math.max(mode.getHeight(), 1);

            // Fuerza a LWJGL a reaplicar el estilo (con/sin borde) en caliente.
            Display.setResizable(false);
            Display.setResizable(true);

            if (grabbed) {
                Mouse.setGrabbed(true);
            }
            ModConfig.windowedActive = this.fullscreen;
            return true;
        } catch (Throwable t) {
            // Si algo falla, revertimos el flag y dejamos que actúe el vanilla.
            this.fullscreen = !this.fullscreen;
            try {
                if (grabbed) {
                    Mouse.setGrabbed(true);
                }
            } catch (Throwable ignored) {
            }
            System.err.println("[Paraguacraft] Windowed fullscreen falló: " + t);
            return false;
        }
    }

    private static void centerWindow(DisplayMode mode) {
        try {
            java.awt.Dimension screen = java.awt.Toolkit.getDefaultToolkit().getScreenSize();
            int x = (screen.width - mode.getWidth()) / 2;
            int y = (screen.height - mode.getHeight()) / 2;
            Display.setLocation(Math.max(x, 0), Math.max(y, 0));
        } catch (Throwable ignored) {
        }
    }

    private static boolean isWindows() {
        String os = System.getProperty("os.name");
        return os != null && os.toLowerCase().contains("win");
    }
}
