package com.paraguacraft.pvp.animations;

import net.minecraft.client.Minecraft;
import net.minecraft.client.entity.EntityPlayerSP;
import net.minecraft.client.renderer.GlStateManager;
import net.minecraft.client.renderer.ItemRenderer;
import net.minecraft.client.renderer.block.model.ItemCameraTransforms;
import net.minecraft.item.EnumAction;
import net.minecraft.item.ItemStack;
import net.minecraft.util.MathHelper;

public class CustomItemRenderer extends ItemRenderer {

    private final Minecraft mc;

    public CustomItemRenderer(Minecraft mcIn) {
        super(mcIn);
        this.mc = mcIn;
    }

    @Override
    public void renderItemInFirstPerson(float partialTicks) {
        // Solución a los 3 errores: Usamos EntityPlayerSP en lugar de AbstractClientPlayer
        EntityPlayerSP player = this.mc.thePlayer;
        float swingProgress = player.getSwingProgress(partialTicks);
        
        ItemStack itemToRender = player.inventory.getCurrentItem();

        // Si la espada existe, se puede bloquear con ella, y el jugador mantiene el click derecho:
        if (itemToRender != null && itemToRender.getItemUseAction() == EnumAction.BLOCK && player.getItemInUseCount() > 0) {
            
            // Limpiamos el buffer y abrimos una matriz aislada súper ligera
            GlStateManager.clear(256);
            GlStateManager.pushMatrix();
            
            // Inyectamos la fluidez pura del Blockhit 1.7.10
            doBlockhitTransformations(swingProgress);
            
            // Renderizamos el ítem en pantalla
            this.renderItem(player, itemToRender, ItemCameraTransforms.TransformType.FIRST_PERSON);
            
            GlStateManager.popMatrix();
        } else {
            // Si solo estás corriendo o pegando sin bloquear, dejamos que el motor Vanilla haga su magia
            super.renderItemInFirstPerson(partialTicks);
        }
    }

    // Matemáticas estáticas: Cero dependencias de variables externas para evitar tirones
    private void doBlockhitTransformations(float swingProgress) {
        // Posicionamiento de la espada en la mano
        GlStateManager.translate(0.56F, -0.52F, -0.71999997F);
        GlStateManager.translate(0.0F, 0.0F, 0.0F); 
        GlStateManager.rotate(45.0F, 0.0F, 1.0F, 0.0F);

        // Curva del swing al clickear izquierdo
        float f = MathHelper.sin(swingProgress * swingProgress * (float)Math.PI);
        float f1 = MathHelper.sin(MathHelper.sqrt_float(swingProgress) * (float)Math.PI);
        
        GlStateManager.rotate(f * -20.0F, 0.0F, 1.0F, 0.0F);
        GlStateManager.rotate(f1 * -20.0F, 0.0F, 0.0F, 1.0F);
        GlStateManager.rotate(f1 * -80.0F, 1.0F, 0.0F, 0.0F);
        GlStateManager.scale(0.4F, 0.4F, 0.4F);
        
        // Rotación de la espada cruzada (El "Bloqueo")
        GlStateManager.translate(-0.5F, 0.2F, 0.0F);
        GlStateManager.rotate(30.0F, 0.0F, 1.0F, 0.0F);
        GlStateManager.rotate(-80.0F, 1.0F, 0.0F, 0.0F);
        GlStateManager.rotate(60.0F, 0.0F, 1.0F, 0.0F);
    }
}