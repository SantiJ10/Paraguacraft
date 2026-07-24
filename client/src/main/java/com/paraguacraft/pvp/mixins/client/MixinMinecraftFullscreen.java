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

    @Shadow
    private void resize(int width, int height) {
        throw new AssertionError();
    }

    /**
     * Self-heal del viewport: si el tamaño real de la ventana (Display) no
     * coincide con el framebuffer del juego, forzamos un resize. Corrige la
     * pantalla renderizada "en la esquina" (cuadrante) que puede pasar con el
     * borderless o con escalado DPI de Windows.
     */
    @Inject(method = "runTick", at = @At("HEAD"))
    private void paraguacraft$healViewport(CallbackInfo ci) {
        if (!ModConfig.windowedFullscreen && !ModConfig.windowedActive) {
            return;
        }
        try {
            int w = Display.getWidth();
            int h = Display.getHeight();
            if (w > 0 && h > 0 && (w != this.displayWidth || h != this.displayHeight)) {
                this.displayWidth = w;
                this.displayHeight = h;
                this.resize(w, h);
            }
        } catch (Throwable ignored) {
        }
    }

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

            // NUNCA fullscreen exclusivo: Discord puede listar/capturar la ventana.
            Display.setFullscreen(false);

            this.displayWidth = Math.max(mode.getWidth(), 1);
            this.displayHeight = Math.max(mode.getHeight(), 1);

            // Un solo toggle de resizable (doble pelea con Discord Overlay).
            try {
                Display.setResizable(true);
            } catch (Throwable ignored) {
            }

            if (grabbed) {
                Mouse.setGrabbed(true);
            }

            try {
                Display.update();
                this.resize(this.displayWidth, this.displayHeight);
            } catch (Throwable ignored) {
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
