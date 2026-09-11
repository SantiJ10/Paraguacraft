const KEY = "paraguacraft.bedrock.lastPlayed";

export function markBedrockPlayed() {
  try {
    localStorage.setItem(KEY, new Date().toISOString());
  } catch {
    /* ignore */
  }
}

export function bedrockLastPlayed(): string | null {
  try {
    return localStorage.getItem(KEY);
  } catch {
    return null;
  }
}
