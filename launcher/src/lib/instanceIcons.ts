import bedrock from "@/assets/instance-icons/bedrock.svg";

import chest from "@/assets/instance-icons/chest.svg";

import craftingTable from "@/assets/instance-icons/crafting_table.svg";

import creeper from "@/assets/instance-icons/creeper.svg";

import diamond from "@/assets/instance-icons/diamond.svg";

import dirt from "@/assets/instance-icons/dirt.svg";

import enchantingTable from "@/assets/instance-icons/enchanting_table.svg";

import furnace from "@/assets/instance-icons/furnace.svg";

import steve from "@/assets/instance-icons/steve.svg";

import tnt from "@/assets/instance-icons/tnt.svg";

import { LOADER_ICONS } from "@/lib/loaderIcons";

import { normalizeLoaderId } from "@/lib/loaders";



export type McInstanceIconId =

  | "mc:dirt"

  | "mc:tnt"

  | "mc:enchanting_table"

  | "mc:chest"

  | "mc:furnace"

  | "mc:crafting_table"

  | "mc:creeper"

  | "mc:steve"

  | "mc:diamond"

  | "mc:bedrock";



export type LoaderInstanceIconId = `loader:${string}`;



export type InstanceIconId = McInstanceIconId | LoaderInstanceIconId;



export interface InstanceIconOption {

  id: McInstanceIconId;

  label: string;

  src: string;

}



/** Icono por defecto (bloque de tierra, como el launcher oficial). */

export const DEFAULT_INSTANCE_ICON: McInstanceIconId = "mc:dirt";



export const INSTANCE_ICONS: InstanceIconOption[] = [

  { id: "mc:dirt", label: "Tierra", src: dirt },

  { id: "mc:tnt", label: "TNT", src: tnt },

  { id: "mc:enchanting_table", label: "Mesa de encantamiento", src: enchantingTable },

  { id: "mc:chest", label: "Cofre", src: chest },

  { id: "mc:furnace", label: "Horno", src: furnace },

  { id: "mc:crafting_table", label: "Mesa de crafteo", src: craftingTable },

  { id: "mc:creeper", label: "Creeper", src: creeper },

  { id: "mc:steve", label: "Steve", src: steve },

  { id: "mc:diamond", label: "Diamante", src: diamond },

  { id: "mc:bedrock", label: "Bedrock", src: bedrock },

];



const MC_ICON_SRC: Record<McInstanceIconId, string> = Object.fromEntries(

  INSTANCE_ICONS.map((i) => [i.id, i.src]),

) as Record<McInstanceIconId, string>;



const LOADER_ICON_SRC: Record<string, string> = Object.fromEntries(

  LOADER_ICONS.map((l) => [`loader:${l.id}`, l.src]),

);



/** Emojis legacy del launcher anterior → icono MC. */

const LEGACY_EMOJI: Record<string, McInstanceIconId> = {

  "\u{1F7E9}": "mc:dirt",

  "\u{1F7E2}": "mc:dirt",

  "\u{1F4E6}": "mc:chest",

  "\u{1F525}": "mc:furnace",

  "\u{2694}": "mc:diamond",

  "\u{1F3D7}": "mc:crafting_table",

  "\u{1F319}": "mc:enchanting_table",

  "\u{1F9F1}": "mc:bedrock",

  "\u{26A1}": "mc:tnt",

};



export function iconForLoader(loader: string): LoaderInstanceIconId {

  return `loader:${normalizeLoaderId(loader)}`;

}



export function resolveInstanceIcon(stored: string | null | undefined): string {

  const value = stored?.trim() ?? "";

  if (!value) return iconForLoader("vanilla");

  if (value.startsWith("loader:") && value in LOADER_ICON_SRC) return value;

  if (value.startsWith("mc:") && value in MC_ICON_SRC) return value;

  if (LEGACY_EMOJI[value]) return LEGACY_EMOJI[value];

  return value;

}



export function getInstanceIconSrc(icon: string | null | undefined): string | null {

  const resolved = resolveInstanceIcon(icon);

  if (resolved.startsWith("loader:")) {

    return LOADER_ICON_SRC[resolved] ?? LOADER_ICON_SRC["loader:vanilla"] ?? null;

  }

  if (resolved.startsWith("mc:")) {

    return MC_ICON_SRC[resolved as McInstanceIconId] ?? MC_ICON_SRC[DEFAULT_INSTANCE_ICON];

  }

  return null;

}



export function isMcInstanceIcon(icon: string | null | undefined): boolean {

  return resolveInstanceIcon(icon).startsWith("mc:");

}



export function isLoaderInstanceIcon(icon: string | null | undefined): boolean {

  return resolveInstanceIcon(icon).startsWith("loader:");

}



export function isCustomInstanceIcon(icon: string | null | undefined): boolean {

  return (icon?.trim() ?? "").startsWith("custom:");

}



/** Resolución recomendada para iconos importados (launcher oficial MC). */

export const INSTANCE_ICON_SIZE = 128;

export const INSTANCE_ICON_MIN_SIZE = 64;

