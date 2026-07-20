package com.paraguacraft.pvp.modern.core;

import com.paraguacraft.pvp.modern.config.ModernConfig;
import com.paraguacraft.pvp.modern.gui.theme.TextUtil;
import com.paraguacraft.pvp.modern.gui.theme.UiTheme;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.network.PlayerListEntry;
import net.minecraft.text.Text;
import net.minecraft.text.TextColor;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

/** Resalta jugadores en tab/nametag por coincidencia parcial de nick. */
public final class NickFinderManager {

    private NickFinderManager() {}

    public static boolean isActive() {
        return ModernConfig.nickFinderEnabled && query().length() >= 2;
    }

    public static String query() {
        return ModernConfig.nickFinderQuery == null ? "" : ModernConfig.nickFinderQuery.trim();
    }

    public static void setQuery(String q) {
        ModernConfig.nickFinderQuery = q == null ? "" : q.trim();
    }

    public static boolean matches(String playerName) {
        if (!isActive() || playerName == null) {
            return false;
        }
        String clean = TextUtil.sanitizeForMcFont(playerName).toLowerCase(Locale.ROOT);
        String q = query().toLowerCase(Locale.ROOT);
        return clean.contains(q);
    }

    /** Nametag/tab: texto cian en negrita si coincide la busqueda. */
    public static Text highlightLabel(Text source) {
        if (!isActive() || source == null) {
            return source;
        }
        String plain = TextUtil.sanitizeForMcFont(source.getString());
        if (!matches(plain)) {
            return source;
        }
        int rgb = UiTheme.accent() & 0xFFFFFF;
        return Text.literal(plain).styled(s -> s.withColor(TextColor.fromRgb(rgb)).withBold(true));
    }

    public static List<PlayerListEntry> findEntries() {
        List<PlayerListEntry> out = new ArrayList<>();
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.getNetworkHandler() == null || !isActive()) {
            return out;
        }
        for (PlayerListEntry entry : client.getNetworkHandler().getPlayerList()) {
            if (entry.getProfile() == null) {
                continue;
            }
            String name = entry.getProfile().name();
            if (matches(name)) {
                out.add(entry);
            }
        }
        out.sort((a, b) -> a.getProfile().name().compareToIgnoreCase(b.getProfile().name()));
        return out;
    }
}
