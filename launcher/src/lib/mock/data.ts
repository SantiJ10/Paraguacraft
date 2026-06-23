import type {
  Account,
  HardwareInfo,
  Instance,
  LoaderInfo,
  MinecraftVersion,
  StoreItem,
} from "@/lib/types";

// Datos de ejemplo para Fase 1 (UI). En Fase 2-3 estos los provee el backend Rust.

export const mockHardware: HardwareInfo = {
  ramGb: 16,
  cpuCores: 6,
  cpuThreads: 12,
  cpuName: "AMD Ryzen 5 5600X",
  gpuName: "NVIDIA GeForce RTX 3060",
  os: "Windows",
  arch: "x86_64",
  perfilSugerido: "alta",
  recommendedRamMb: 6144,
  recommendedGc: "ZGC",
};

export const mockAccounts: Account[] = [
  {
    id: "acc-1",
    type: "microsoft",
    username: "ParaguaPro",
    uuid: "069a79f4-44e9-4726-a5be-fca90e38aaf5",
    avatarUrl: "https://crafatar.com/avatars/069a79f4-44e9-4726-a5be-fca90e38aaf5?overlay",
    active: true,
    premium: true,
  },
  {
    id: "acc-2",
    type: "offline",
    username: "Invitado_01",
    uuid: "f84c6a79-0a4e-45e0-879b-cd49ebd4c4e2",
    avatarUrl: "https://crafatar.com/avatars/f84c6a79-0a4e-45e0-879b-cd49ebd4c4e2?overlay",
    active: false,
    premium: false,
  },
];

export const mockInstances: Instance[] = [
  {
    id: "inst-1",
    name: "Paraguacraft PvP",
    icon: "mc:diamond",
    mcVersion: "1.8.9",
    loader: "paraguacraft-pvp",
    loaderVersion: "11.15.1.2318",
    source: "paraguacraft",
    lastPlayed: "2026-06-18T22:10:00Z",
    totalPlayMinutes: 4820,
    ramMb: 4096,
    modCount: 2,
  },
  {
    id: "inst-2",
    name: "Fabric + Iris 1.21.1",
    icon: "mc:enchanting_table",
    mcVersion: "1.21.1",
    loader: "fabric-iris",
    loaderVersion: "0.16.5",
    source: "paraguacraft",
    lastPlayed: "2026-06-17T18:40:00Z",
    totalPlayMinutes: 1290,
    ramMb: 6144,
    modCount: 8,
  },
  {
    id: "inst-3",
    name: "All The Mods 9",
    icon: "mc:bedrock",
    mcVersion: "1.20.1",
    loader: "neoforge",
    loaderVersion: "47.1.106",
    source: "curseforge",
    lastPlayed: "2026-06-10T12:00:00Z",
    totalPlayMinutes: 320,
    ramMb: 8192,
    modCount: 412,
  },
  {
    id: "inst-4",
    name: "Vanilla 1.21.4",
    icon: "mc:dirt",
    mcVersion: "1.21.4",
    loader: "vanilla",
    loaderVersion: "-",
    source: "vanilla",
    lastPlayed: null,
    totalPlayMinutes: 0,
    ramMb: 4096,
    modCount: 0,
  },
];

export const mockVersions: MinecraftVersion[] = [
  { id: "1.21.4", channel: "release", releaseDate: "2024-12-03", installed: true },
  { id: "1.21.1", channel: "release", releaseDate: "2024-08-08", installed: true },
  { id: "1.20.1", channel: "release", releaseDate: "2023-06-12", installed: true },
  { id: "1.16.5", channel: "release", releaseDate: "2021-01-15", installed: false },
  { id: "1.8.9", channel: "release", releaseDate: "2015-12-09", installed: true },
  { id: "25w06a", channel: "snapshot", releaseDate: "2025-02-05", installed: false },
  { id: "b1.7.3", channel: "old_beta", releaseDate: "2011-07-08", installed: false },
  { id: "a1.2.6", channel: "old_alpha", releaseDate: "2010-12-03", installed: false },
];

export const mockLoaders: LoaderInfo[] = [
  { id: "vanilla", name: "Vanilla", description: "Sin mods, juego oficial.", versions: ["-"] },
  { id: "fabric", name: "Fabric", description: "Loader liviano y rapido.", versions: ["0.16.5", "0.16.4", "0.15.11"] },
  { id: "fabric-iris", name: "Fabric + Iris", description: "Fabric con Sodium + Iris (shaders).", versions: ["0.16.5", "0.16.4"] },
  { id: "forge", name: "Forge", description: "El loader de mods mas usado.", versions: ["52.0.10", "51.0.33", "47.3.0"] },
  { id: "neoforge", name: "NeoForge", description: "Fork moderno de Forge.", versions: ["21.1.106", "20.4.237"] },
  { id: "quilt", name: "Quilt", description: "Fork de Fabric con extras.", versions: ["0.27.1", "0.26.4"] },
  { id: "optifine", name: "OptiFine", description: "Optimizacion + shaders clasicos.", versions: ["HD U I7", "HD U I6"] },
];

const titles: Array<[string, StoreItem["projectType"], StoreItem["provider"], string]> = [
  ["Sodium", "mod", "modrinth", "Rendimiento de render extremo."],
  ["Iris Shaders", "mod", "modrinth", "Soporte de shaders para Fabric."],
  ["Fabric API", "mod", "modrinth", "Dependencia base de mods Fabric."],
  ["Complementary Shaders", "shader", "modrinth", "Shaders pulidos y rapidos."],
  ["Faithful 32x", "resourcepack", "modrinth", "Texturas vanilla en alta resolucion."],
  ["All The Mods 9", "modpack", "curseforge", "Modpack kitchen-sink gigante."],
  ["JEI", "mod", "curseforge", "Just Enough Items: recetas in-game."],
  ["Distant Horizons", "mod", "modrinth", "LOD: render a distancias enormes."],
  ["BSL Shaders", "shader", "curseforge", "Shaders cinematograficos."],
  ["Terralith", "datapack", "modrinth", "Generacion de mundo mejorada."],
  ["EssentialsX", "plugin", "modrinth", "Comandos esenciales de servidor."],
  ["Create", "mod", "curseforge", "Mecanica y automatizacion."],
];

export const mockStoreItems: StoreItem[] = titles.map(([title, projectType, provider, desc], i) => ({
  id: `item-${i}`,
  slug: title.toLowerCase().replace(/[^a-z0-9]+/g, "-"),
  title,
  author: ["jellysquid3", "coderbot", "shedaniel", "kingbdogz"][i % 4],
  description: desc,
  iconUrl: "",
  downloads: Math.floor(50_000 + Math.random() * 80_000_000),
  follows: Math.floor(1_000 + Math.random() * 500_000),
  projectType,
  provider,
  categories: ["optimization", "utility", "adventure"].slice(0, (i % 3) + 1),
}));
