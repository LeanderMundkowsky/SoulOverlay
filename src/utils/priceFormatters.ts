import type { PriceEntry } from "@/bindings";

export function formatScu(val: number | undefined): string {
  if (!val || val === 0) return "-";
  if (val >= 1000) return (val / 1000).toFixed(1).replace(/\.0$/, "") + "K";
  return val.toLocaleString("en-US", { maximumFractionDigits: 0 });
}

export function formatPrice(val: number | undefined): string {
  if (!val || val === 0) return "-";
  return val.toLocaleString("en-US", { minimumFractionDigits: 0, maximumFractionDigits: 0 });
}

export function formatSimplePrice(val: number): string {
  if (val === 0) return "-";
  return val.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

export function inventoryPercent(entry: PriceEntry): number {
  if (!entry.scu_max || entry.scu_max <= 0) return 0;
  return Math.min(100, Math.round(((entry.scu_last ?? 0) / entry.scu_max) * 100));
}

export function inventoryBarColor(pct: number): string {
  if (pct >= 80) return "bg-green-400/50";
  if (pct >= 40) return "bg-yellow-400/50";
  if (pct > 0) return "bg-blue-400/50";
  return "bg-white/10";
}

export function relativeAge(timestamp: string): string {
  if (!timestamp) return "-";
  const epoch = parseInt(timestamp, 10);
  if (isNaN(epoch)) return "-";
  const now = Math.floor(Date.now() / 1000);
  const diff = now - epoch;
  if (diff < 60) return "<1m";
  if (diff < 3600) return Math.floor(diff / 60) + "m";
  if (diff < 86400) return Math.floor(diff / 3600) + "h";
  return Math.floor(diff / 86400) + "d";
}

export function shortSystem(system: string | undefined): string {
  if (!system) return "—";
  const map: Record<string, string> = {
    Stanton: "ST",
    Pyro: "PY",
    Nyx: "NY",
  };
  return map[system] ?? system.substring(0, 3).toUpperCase();
}

export function shortFaction(faction: string | undefined): string {
  if (!faction) return "—";
  const map: Record<string, string> = {
    "United Empire of Earth": "UEE",
    "Citizens for Prosperity": "CitPro",
    "Rest & Relax": "R&R",
  };
  return map[faction] ?? faction;
}

export function shortTerminal(terminal: string): string {
  return terminal.replace(/^Admin - /, "");
}

export function avgOf(entries: PriceEntry[], key: keyof PriceEntry): number {
  const vals = entries.map((e) => e[key] as number).filter((v) => v > 0);
  if (vals.length === 0) return 0;
  return vals.reduce((sum, v) => sum + v, 0) / vals.length;
}

export function avgInventoryPercent(entries: PriceEntry[]): number {
  const pcts = entries.map((e) => inventoryPercent(e)).filter((v) => v > 0);
  if (pcts.length === 0) return 0;
  return Math.round(pcts.reduce((sum, v) => sum + v, 0) / pcts.length);
}
