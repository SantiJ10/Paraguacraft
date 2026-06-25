package com.paraguacraft.pvp.resourcepack;

import net.minecraft.client.Minecraft;
import net.minecraft.util.ChatComponentText;

import java.awt.Component;
import java.awt.datatransfer.DataFlavor;
import java.awt.dnd.DropTarget;
import java.awt.dnd.DropTargetAdapter;
import java.awt.dnd.DropTargetDropEvent;
import java.io.File;
import java.util.List;

import org.lwjgl.opengl.Display;

/**
 * Drag &amp; drop de .zip sobre la ventana del juego (AWT + LWJGL Display).
 */
public final class PackDropTarget {

    private static PackDropListener listener;

    public interface PackDropListener {
        void onPackDropped(File file);
    }

    private PackDropTarget() {}

    public static void register(PackDropListener dropListener) {
        listener = dropListener;
        Component parent = Display.getParent();
        if (parent == null) {
            return;
        }
        parent.setDropTarget(new DropTarget(parent, new DropTargetAdapter() {
            @Override
            public void drop(DropTargetDropEvent evt) {
                try {
                    evt.acceptDrop(evt.getDropAction());
                    @SuppressWarnings("unchecked")
                    List<File> files = (List<File>) evt.getTransferable()
                        .getTransferData(DataFlavor.javaFileListFlavor);
                    if (files == null || files.isEmpty()) {
                        evt.dropComplete(false);
                        return;
                    }
                    File file = files.get(0);
                    evt.dropComplete(true);
                    if (listener != null) {
                        listener.onPackDropped(file);
                    }
                } catch (Exception e) {
                    evt.dropComplete(false);
                    notifyError("No se pudo importar el pack arrastrado");
                }
            }
        }));
    }

    public static void unregister() {
        listener = null;
        Component parent = Display.getParent();
        if (parent != null) {
            parent.setDropTarget(null);
        }
    }

    public static void notifyError(String msg) {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer != null) {
            mc.thePlayer.addChatMessage(new ChatComponentText(
                "\u00A78[\u00A79Paraguacraft\u00A78] \u00A7c" + msg));
        }
    }

    public static void notifyOk(String msg) {
        Minecraft mc = Minecraft.getMinecraft();
        if (mc.thePlayer != null) {
            mc.thePlayer.addChatMessage(new ChatComponentText(
                "\u00A78[\u00A79Paraguacraft\u00A78] \u00A7a" + msg));
        }
    }
}
