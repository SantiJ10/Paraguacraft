import chest from "@/assets/instance-icons/chest.svg";
import craftingTable from "@/assets/instance-icons/crafting_table.svg";
import dirt from "@/assets/instance-icons/dirt.svg";
import enchantingTable from "@/assets/instance-icons/enchanting_table.svg";

export type ContentFolderKey = "mods" | "resourcepacks" | "shaderpacks" | "datapacks";

export interface ContentFolderMeta {
  id: ContentFolderKey;
  label: string;
  icon: string;
}

export const CONTENT_FOLDERS: ContentFolderMeta[] = [
  { id: "mods", label: "Mods", icon: craftingTable },
  { id: "resourcepacks", label: "Resource Packs", icon: dirt },
  { id: "shaderpacks", label: "Shaders", icon: enchantingTable },
  { id: "datapacks", label: "Data Packs", icon: chest },
];

export function contentFolderIcon(folder: string): string {
  const hit = CONTENT_FOLDERS.find((f) => f.id === folder);
  return hit?.icon ?? chest;
}

export function contentFolderLabel(folder: string): string {
  const hit = CONTENT_FOLDERS.find((f) => f.id === folder);
  return hit?.label ?? folder;
}
