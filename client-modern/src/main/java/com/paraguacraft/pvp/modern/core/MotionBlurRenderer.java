package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import net.minecraft.client.MinecraftClient;
import org.lwjgl.opengl.GL11;
import org.lwjgl.opengl.GL20;
import org.lwjgl.opengl.GL30;

/**
 * Motion blur por accumulation FBO (mezcla el color buffer actual con el frame anterior).
 * No usa el pipeline de post-process de Iris: si hay shader pack activo, se omite.
 */
public final class MotionBlurRenderer {

    private static final String VERT = "#version 330 core\n"
        + "out vec2 uv;\n"
        + "void main(){\n"
        + "  vec2 p = vec2((gl_VertexID << 1) & 2, gl_VertexID & 2);\n"
        + "  uv = p;\n"
        + "  gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);\n"
        + "}\n";

    private static final String FRAG = "#version 330 core\n"
        + "in vec2 uv;\n"
        + "uniform sampler2D prevTex;\n"
        + "uniform float mixFactor;\n"
        + "out vec4 frag;\n"
        + "void main(){\n"
        + "  vec4 p = texture(prevTex, vec2(uv.x, uv.y));\n"
        + "  frag = vec4(p.rgb, mixFactor);\n"
        + "}\n";

    private static int fbo;
    private static int tex;
    private static int program;
    private static int vao;
    private static int lastW;
    private static int lastH;
    private static boolean primed;
    private static boolean failed;

    private MotionBlurRenderer() {}

    public static void afterWorld() {
        if (failed || !ModernConfig.motionBlurEnabled || ModernConfig.motionBlurAmount <= 0.001f) {
            primed = false;
            return;
        }
        MinecraftClient mc = MinecraftClient.getInstance();
        if (mc == null || mc.world == null || mc.currentScreen != null) {
            primed = false;
            return;
        }
        if (ShaderAutoManager.shadersInUse()) {
            primed = false;
            return;
        }
        int w = mc.getWindow().getFramebufferWidth();
        int h = mc.getWindow().getFramebufferHeight();
        if (w <= 0 || h <= 0) {
            return;
        }
        try {
            ensureGl(w, h);
            float mix = blendFactor();
            int gameFbo = GL11.glGetInteger(GL30.GL_FRAMEBUFFER_BINDING);
            int prevTex = GL11.glGetInteger(GL11.GL_TEXTURE_BINDING_2D);
            int prevProg = GL11.glGetInteger(GL20.GL_CURRENT_PROGRAM);
            int prevVao = GL11.glGetInteger(GL30.GL_VERTEX_ARRAY_BINDING);
            boolean blend = GL11.glIsEnabled(GL11.GL_BLEND);
            boolean depth = GL11.glIsEnabled(GL11.GL_DEPTH_TEST);

            if (primed) {
                GL30.glBindFramebuffer(GL30.GL_FRAMEBUFFER, gameFbo);
                GL11.glDisable(GL11.GL_DEPTH_TEST);
                GL11.glEnable(GL11.GL_BLEND);
                GL11.glBlendFunc(GL11.GL_SRC_ALPHA, GL11.GL_ONE_MINUS_SRC_ALPHA);
                GL20.glUseProgram(program);
                GL20.glUniform1i(GL20.glGetUniformLocation(program, "prevTex"), 0);
                GL20.glUniform1f(GL20.glGetUniformLocation(program, "mixFactor"), mix);
                GL11.glBindTexture(GL11.GL_TEXTURE_2D, tex);
                GL30.glBindVertexArray(vao);
                GL11.glDrawArrays(GL11.GL_TRIANGLES, 0, 3);
            }

            GL30.glBindFramebuffer(GL30.GL_READ_FRAMEBUFFER, gameFbo);
            GL30.glBindFramebuffer(GL30.GL_DRAW_FRAMEBUFFER, fbo);
            GL11.glDisable(GL11.GL_BLEND);
            GL30.glBlitFramebuffer(0, 0, w, h, 0, 0, w, h, GL11.GL_COLOR_BUFFER_BIT, GL11.GL_NEAREST);
            primed = true;

            GL30.glBindFramebuffer(GL30.GL_FRAMEBUFFER, gameFbo);
            GL20.glUseProgram(prevProg);
            GL11.glBindTexture(GL11.GL_TEXTURE_2D, prevTex);
            GL30.glBindVertexArray(prevVao);
            if (depth) {
                GL11.glEnable(GL11.GL_DEPTH_TEST);
            } else {
                GL11.glDisable(GL11.GL_DEPTH_TEST);
            }
            if (blend) {
                GL11.glEnable(GL11.GL_BLEND);
            } else {
                GL11.glDisable(GL11.GL_BLEND);
            }
        } catch (Throwable t) {
            failed = true;
            System.err.println("[paraguacraft] motion blur disabled: " + t.getMessage());
        }
    }

    private static float blendFactor() {
        float a = Math.max(0.0F, Math.min(1.0F, ModernConfig.motionBlurAmount));
        if (ModernConfig.motionBlurType == 1) {
            return a * a * 0.85F + a * 0.1F;
        }
        return a * 0.85F;
    }

    private static void ensureGl(int w, int h) {
        if (program == 0) {
            program = link(VERT, FRAG);
            vao = GL30.glGenVertexArrays();
        }
        if (fbo != 0 && lastW == w && lastH == h) {
            return;
        }
        if (tex != 0) {
            GL11.glDeleteTextures(tex);
        }
        if (fbo != 0) {
            GL30.glDeleteFramebuffers(fbo);
        }
        tex = GL11.glGenTextures();
        GL11.glBindTexture(GL11.GL_TEXTURE_2D, tex);
        GL11.glTexParameteri(GL11.GL_TEXTURE_2D, GL11.GL_TEXTURE_MIN_FILTER, GL11.GL_LINEAR);
        GL11.glTexParameteri(GL11.GL_TEXTURE_2D, GL11.GL_TEXTURE_MAG_FILTER, GL11.GL_LINEAR);
        GL11.glTexParameteri(GL11.GL_TEXTURE_2D, GL11.GL_TEXTURE_WRAP_S, 0x812F);
        GL11.glTexParameteri(GL11.GL_TEXTURE_2D, GL11.GL_TEXTURE_WRAP_T, 0x812F);
        GL11.glTexImage2D(GL11.GL_TEXTURE_2D, 0, GL11.GL_RGBA8, w, h, 0, GL11.GL_RGBA, GL11.GL_UNSIGNED_BYTE, 0L);
        fbo = GL30.glGenFramebuffers();
        GL30.glBindFramebuffer(GL30.GL_FRAMEBUFFER, fbo);
        GL30.glFramebufferTexture2D(GL30.GL_FRAMEBUFFER, GL30.GL_COLOR_ATTACHMENT0, GL11.GL_TEXTURE_2D, tex, 0);
        lastW = w;
        lastH = h;
        primed = false;
    }

    private static int link(String vertSrc, String fragSrc) {
        int vs = compile(GL20.GL_VERTEX_SHADER, vertSrc);
        int fs = compile(GL20.GL_FRAGMENT_SHADER, fragSrc);
        int prog = GL20.glCreateProgram();
        GL20.glAttachShader(prog, vs);
        GL20.glAttachShader(prog, fs);
        GL20.glLinkProgram(prog);
        GL20.glDeleteShader(vs);
        GL20.glDeleteShader(fs);
        if (GL20.glGetProgrami(prog, GL20.GL_LINK_STATUS) == GL11.GL_FALSE) {
            throw new IllegalStateException(GL20.glGetProgramInfoLog(prog));
        }
        return prog;
    }

    private static int compile(int type, String src) {
        int sh = GL20.glCreateShader(type);
        GL20.glShaderSource(sh, src);
        GL20.glCompileShader(sh);
        if (GL20.glGetShaderi(sh, GL20.GL_COMPILE_STATUS) == GL11.GL_FALSE) {
            throw new IllegalStateException(GL20.glGetShaderInfoLog(sh));
        }
        return sh;
    }
}
