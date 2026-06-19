import type { GcType, HardwareInfo, HardwareTier } from "@/lib/types";

// Autoconfig de JVM/GC/RAM derivada del hardware. En Fase 2 la version
// autoritativa la calcula Rust; esta es la version cliente para previsualizar
// recomendaciones en el wizard y ajustes.

export interface JvmRecommendation {
  ramMb: number;
  gc: GcType;
  tier: HardwareTier;
  flags: string[];
}

export function recommendForHardware(hw: HardwareInfo): JvmRecommendation {
  const totalMb = Math.round(hw.ramGb * 1024);
  // Reservamos memoria para el SO; nunca asignamos mas del 60% en gama baja.
  let ramMb: number;
  if (hw.ramGb <= 8) ramMb = Math.min(4096, Math.floor(totalMb * 0.45));
  else if (hw.ramGb <= 16) ramMb = 6144;
  else ramMb = 8192;
  ramMb = Math.max(2048, Math.round(ramMb / 512) * 512);

  const gc: GcType = hw.ramGb >= 8 ? "ZGC" : "G1GC";

  const flags =
    gc === "ZGC"
      ? ["-XX:+UseZGC", "-XX:+DisableExplicitGC", "-XX:+AlwaysPreTouch"]
      : [
          "-XX:+UseG1GC",
          "-XX:G1NewSizePercent=30",
          "-XX:G1ReservePercent=20",
          "-XX:MaxGCPauseMillis=50",
          "-XX:+DisableExplicitGC",
        ];

  return { ramMb, gc, tier: hw.perfilSugerido, flags };
}

export function tierLabel(tier: HardwareTier): string {
  return { baja: "Gama baja", media: "Gama media", alta: "Gama alta" }[tier];
}
