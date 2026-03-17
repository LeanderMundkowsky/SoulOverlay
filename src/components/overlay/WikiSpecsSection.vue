<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import { useWiki } from "@/composables/useWiki";
import type { WikiEntitySpecs } from "@/bindings";

const props = defineProps<{
  entityId: string;
  entityKind: string;
  entityName: string;
  uuid?: string;
}>();

const { wikiSpecs, wikiSpecsLoading, wikiSpecsError, getWikiSpecs } = useWiki();

const expanded = ref(false);

function fetchSpecs() {
  getWikiSpecs(props.entityKind, props.entityId, props.entityName, props.uuid ?? "");
}

onMounted(() => { fetchSpecs(); });
watch(() => props.entityId, () => { fetchSpecs(); });

function fmt(val: number | null | undefined, decimals = 1, suffix = ""): string {
  if (val == null) return "";
  return val.toLocaleString("en-US", { minimumFractionDigits: 0, maximumFractionDigits: decimals }) + suffix;
}

function fmtBig(val: number | null | undefined): string {
  if (val == null) return "";
  if (val >= 1_000_000) return (val / 1_000_000).toFixed(1) + "M";
  if (val >= 1_000) return (val / 1_000).toFixed(1) + "K";
  return val.toFixed(0);
}

// Determine which curated stats to show based on entity kind and classification
function isPowerPlant(s: WikiEntitySpecs): boolean {
  return s.classification?.toLowerCase().includes("power_plant") === true
    || s.item_type?.toLowerCase().includes("power") === true;
}
function isWeapon(s: WikiEntitySpecs): boolean {
  return s.weapon_class != null || s.weapon_type != null || s.dps_physical != null;
}
function isShield(s: WikiEntitySpecs): boolean {
  return s.classification?.toLowerCase().includes("shield") === true;
}
function isQuantumDrive(s: WikiEntitySpecs): boolean {
  return s.classification?.toLowerCase().includes("quantum") === true
    || s.item_type?.toLowerCase().includes("quantum") === true;
}
function isCooler(s: WikiEntitySpecs): boolean {
  return s.classification?.toLowerCase().includes("cooler") === true;
}
function isVehicle(): boolean {
  return props.entityKind === "vehicle" || props.entityKind === "ground vehicle";
}

// Check if we have any stats to display
function hasStats(s: WikiEntitySpecs): boolean {
  return s.health != null || s.power_output != null || s.power_draw != null
    || s.dps_physical != null || s.dps_energy != null || s.damage_per_shot != null
    || s.scm_speed != null || s.max_speed != null || s.shield_hp != null
    || s.quantum_speed != null || s.cargo_capacity != null
    || s.mass != null || s.em_max != null || s.ir_emission != null
    || s.cooling_rate_max != null || s.weapon_range != null
    || s.armor_health != null;
}
</script>

<template>
  <!-- Loading -->
  <div v-if="wikiSpecsLoading" class="bg-[#1a1d24] border border-white/10 rounded-xl px-4 py-3 flex justify-center">
    <LoadingSpinner text="Loading wiki specs..." />
  </div>

  <!-- Error / no data: silently hide -->
  <template v-else-if="wikiSpecsError || !wikiSpecs || !hasStats(wikiSpecs)" />

  <!-- Wiki specs display -->
  <div v-else class="bg-[#1a1d24] border border-white/10 rounded-xl overflow-hidden">
    <!-- Header -->
    <div class="px-4 py-2 border-b border-white/5 flex items-center gap-2">
      <span class="text-white/50 text-xs uppercase tracking-widest font-medium">Wiki Specs</span>
      <span v-if="wikiSpecs.game_version" class="text-white/20 text-xs">v{{ wikiSpecs.game_version }}</span>
      <div class="flex-1" />
      <button
        v-if="wikiSpecs.wiki_url"
        @click="openUrl(wikiSpecs.wiki_url!)"
        class="text-teal-400/50 hover:text-teal-400 text-xs transition-colors cursor-pointer"
      >Wiki ↗</button>
    </div>

    <!-- Description (if available and source is wiki-only) -->
    <div v-if="wikiSpecs.description" class="px-4 py-2 border-b border-white/5">
      <p class="text-white/50 text-xs leading-relaxed line-clamp-2">{{ wikiSpecs.description }}</p>
    </div>

    <!-- Curated key stats -->
    <div class="px-4 py-3 space-y-2">
      <!-- Power Plant stats -->
      <div v-if="isPowerPlant(wikiSpecs)" class="grid grid-cols-3 gap-x-4 gap-y-1.5 text-xs">
        <div v-if="wikiSpecs.power_output"><span class="text-white/40">Power</span> <span class="text-amber-400">{{ fmtBig(wikiSpecs.power_output) }}</span></div>
        <div v-if="wikiSpecs.health"><span class="text-white/40">HP</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.health) }}</span></div>
        <div v-if="wikiSpecs.em_max"><span class="text-white/40">EM max</span> <span class="text-white/70">{{ fmt(wikiSpecs.em_max) }}</span></div>
        <div v-if="wikiSpecs.ir_emission"><span class="text-white/40">IR</span> <span class="text-white/70">{{ fmt(wikiSpecs.ir_emission) }}</span></div>
        <div v-if="wikiSpecs.cooling_rate_max"><span class="text-white/40">Cooling</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.cooling_rate_max) }}/s</span></div>
        <div v-if="wikiSpecs.max_temp"><span class="text-white/40">Max temp</span> <span class="text-white/70">{{ fmt(wikiSpecs.max_temp, 0) }}°</span></div>
      </div>

      <!-- Weapon stats -->
      <div v-else-if="isWeapon(wikiSpecs)" class="grid grid-cols-3 gap-x-4 gap-y-1.5 text-xs">
        <div v-if="wikiSpecs.dps_physical || wikiSpecs.dps_energy">
          <span class="text-white/40">DPS</span>
          <span class="text-red-400">{{ fmtBig(wikiSpecs.dps_physical ?? wikiSpecs.dps_energy ?? 0) }}</span>
        </div>
        <div v-if="wikiSpecs.damage_per_shot"><span class="text-white/40">Alpha</span> <span class="text-white/70">{{ fmt(wikiSpecs.damage_per_shot) }}</span></div>
        <div v-if="wikiSpecs.weapon_range"><span class="text-white/40">Range</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.weapon_range) }}m</span></div>
        <div v-if="wikiSpecs.weapon_rpm"><span class="text-white/40">RPM</span> <span class="text-white/70">{{ fmt(wikiSpecs.weapon_rpm, 0) }}</span></div>
        <div v-if="wikiSpecs.ammo_speed"><span class="text-white/40">Velocity</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.ammo_speed) }} m/s</span></div>
        <div v-if="wikiSpecs.weapon_type"><span class="text-white/40">Type</span> <span class="text-white/70">{{ wikiSpecs.weapon_type }}</span></div>
      </div>

      <!-- Shield stats -->
      <div v-else-if="isShield(wikiSpecs)" class="grid grid-cols-3 gap-x-4 gap-y-1.5 text-xs">
        <div v-if="wikiSpecs.shield_hp"><span class="text-white/40">Shield HP</span> <span class="text-cyan-400">{{ fmtBig(wikiSpecs.shield_hp) }}</span></div>
        <div v-if="wikiSpecs.shield_regen"><span class="text-white/40">Regen</span> <span class="text-white/70">{{ fmt(wikiSpecs.shield_regen) }}/s</span></div>
        <div v-if="wikiSpecs.health"><span class="text-white/40">HP</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.health) }}</span></div>
        <div v-if="wikiSpecs.power_draw"><span class="text-white/40">Power</span> <span class="text-white/70">{{ fmt(wikiSpecs.power_draw) }}</span></div>
        <div v-if="wikiSpecs.em_max"><span class="text-white/40">EM max</span> <span class="text-white/70">{{ fmt(wikiSpecs.em_max) }}</span></div>
      </div>

      <!-- Quantum drive stats -->
      <div v-else-if="isQuantumDrive(wikiSpecs)" class="grid grid-cols-3 gap-x-4 gap-y-1.5 text-xs">
        <div v-if="wikiSpecs.quantum_speed"><span class="text-white/40">QT Speed</span> <span class="text-purple-400">{{ fmtBig(wikiSpecs.quantum_speed) }} m/s</span></div>
        <div v-if="wikiSpecs.quantum_range"><span class="text-white/40">Range</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.quantum_range) }} m</span></div>
        <div v-if="wikiSpecs.quantum_spool_time"><span class="text-white/40">Spool</span> <span class="text-white/70">{{ fmt(wikiSpecs.quantum_spool_time) }}s</span></div>
        <div v-if="wikiSpecs.quantum_fuel_capacity"><span class="text-white/40">Fuel</span> <span class="text-white/70">{{ fmt(wikiSpecs.quantum_fuel_capacity) }}</span></div>
        <div v-if="wikiSpecs.health"><span class="text-white/40">HP</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.health) }}</span></div>
        <div v-if="wikiSpecs.power_draw"><span class="text-white/40">Power</span> <span class="text-white/70">{{ fmt(wikiSpecs.power_draw) }}</span></div>
      </div>

      <!-- Cooler stats -->
      <div v-else-if="isCooler(wikiSpecs)" class="grid grid-cols-3 gap-x-4 gap-y-1.5 text-xs">
        <div v-if="wikiSpecs.cooling_rate_max"><span class="text-white/40">Cooling</span> <span class="text-cyan-400">{{ fmtBig(wikiSpecs.cooling_rate_max) }}/s</span></div>
        <div v-if="wikiSpecs.health"><span class="text-white/40">HP</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.health) }}</span></div>
        <div v-if="wikiSpecs.power_draw"><span class="text-white/40">Power</span> <span class="text-white/70">{{ fmt(wikiSpecs.power_draw) }}</span></div>
        <div v-if="wikiSpecs.em_max"><span class="text-white/40">EM max</span> <span class="text-white/70">{{ fmt(wikiSpecs.em_max) }}</span></div>
        <div v-if="wikiSpecs.ir_emission"><span class="text-white/40">IR</span> <span class="text-white/70">{{ fmt(wikiSpecs.ir_emission) }}</span></div>
      </div>

      <!-- Vehicle stats -->
      <div v-else-if="isVehicle()" class="grid grid-cols-3 gap-x-4 gap-y-1.5 text-xs">
        <div v-if="wikiSpecs.scm_speed"><span class="text-white/40">SCM</span> <span class="text-blue-400">{{ fmt(wikiSpecs.scm_speed, 0) }} m/s</span></div>
        <div v-if="wikiSpecs.max_speed"><span class="text-white/40">Max</span> <span class="text-blue-400">{{ fmt(wikiSpecs.max_speed, 0) }} m/s</span></div>
        <div v-if="wikiSpecs.shield_hp"><span class="text-white/40">Shield</span> <span class="text-cyan-400">{{ fmtBig(wikiSpecs.shield_hp) }}</span></div>
        <div v-if="wikiSpecs.cargo_capacity"><span class="text-white/40">Cargo</span> <span class="text-white/70">{{ fmt(wikiSpecs.cargo_capacity, 0) }} SCU</span></div>
        <div v-if="wikiSpecs.crew_max"><span class="text-white/40">Crew</span> <span class="text-white/70">{{ wikiSpecs.crew_min ?? 1 }}–{{ wikiSpecs.crew_max }}</span></div>
        <div v-if="wikiSpecs.quantum_range"><span class="text-white/40">QT Range</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.quantum_range) }} m</span></div>
        <div v-if="wikiSpecs.health"><span class="text-white/40">Hull HP</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.health) }}</span></div>
        <div v-if="wikiSpecs.mass"><span class="text-white/40">Mass</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.mass) }} kg</span></div>
        <div v-if="wikiSpecs.boost_forward"><span class="text-white/40">Boost</span> <span class="text-white/70">{{ fmt(wikiSpecs.boost_forward, 0) }} m/s</span></div>
      </div>

      <!-- Default / generic item stats -->
      <div v-else class="grid grid-cols-3 gap-x-4 gap-y-1.5 text-xs">
        <div v-if="wikiSpecs.health"><span class="text-white/40">HP</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.health) }}</span></div>
        <div v-if="wikiSpecs.power_draw"><span class="text-white/40">Power</span> <span class="text-white/70">{{ fmt(wikiSpecs.power_draw) }}</span></div>
        <div v-if="wikiSpecs.em_max"><span class="text-white/40">EM max</span> <span class="text-white/70">{{ fmt(wikiSpecs.em_max) }}</span></div>
        <div v-if="wikiSpecs.ir_emission"><span class="text-white/40">IR</span> <span class="text-white/70">{{ fmt(wikiSpecs.ir_emission) }}</span></div>
        <div v-if="wikiSpecs.mass"><span class="text-white/40">Mass</span> <span class="text-white/70">{{ fmt(wikiSpecs.mass, 1) }} kg</span></div>
        <div v-if="wikiSpecs.size != null"><span class="text-white/40">Size</span> <span class="text-white/70">{{ wikiSpecs.size }}</span></div>
      </div>

      <!-- Expand toggle -->
      <button
        @click="expanded = !expanded"
        class="w-full text-center text-white/30 hover:text-white/50 text-xs py-1 transition-colors cursor-pointer"
      >
        {{ expanded ? '▲ Less' : '▼ All specs' }}
      </button>

      <!-- Expanded details -->
      <div v-if="expanded" class="space-y-3 pt-1 border-t border-white/5">
        <!-- Identity -->
        <div v-if="wikiSpecs.manufacturer_name || wikiSpecs.classification || wikiSpecs.grade" class="space-y-1">
          <div class="text-white/30 text-[10px] uppercase tracking-widest">Identity</div>
          <div class="grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
            <div v-if="wikiSpecs.manufacturer_name"><span class="text-white/40">Manufacturer</span> <span class="text-white/70">{{ wikiSpecs.manufacturer_name }}</span></div>
            <div v-if="wikiSpecs.classification"><span class="text-white/40">Class</span> <span class="text-white/70">{{ wikiSpecs.classification }}</span></div>
            <div v-if="wikiSpecs.grade"><span class="text-white/40">Grade</span> <span class="text-white/70">{{ wikiSpecs.grade }}</span></div>
            <div v-if="wikiSpecs.item_type"><span class="text-white/40">Type</span> <span class="text-white/70">{{ wikiSpecs.item_type }}</span></div>
            <div v-if="wikiSpecs.sub_type"><span class="text-white/40">Sub-type</span> <span class="text-white/70">{{ wikiSpecs.sub_type }}</span></div>
          </div>
        </div>

        <!-- Durability / Resistance -->
        <div v-if="wikiSpecs.health || wikiSpecs.resist_physical" class="space-y-1">
          <div class="text-white/30 text-[10px] uppercase tracking-widest">Durability</div>
          <div class="grid grid-cols-3 gap-x-4 gap-y-1 text-xs">
            <div v-if="wikiSpecs.health"><span class="text-white/40">HP</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.health) }}</span></div>
            <div v-if="wikiSpecs.resist_physical"><span class="text-white/40">Phys</span> <span class="text-white/70">{{ fmt(wikiSpecs.resist_physical, 2) }}</span></div>
            <div v-if="wikiSpecs.resist_energy"><span class="text-white/40">Energy</span> <span class="text-white/70">{{ fmt(wikiSpecs.resist_energy, 2) }}</span></div>
            <div v-if="wikiSpecs.resist_thermal"><span class="text-white/40">Therm</span> <span class="text-white/70">{{ fmt(wikiSpecs.resist_thermal, 2) }}</span></div>
            <div v-if="wikiSpecs.resist_distortion"><span class="text-white/40">Dist</span> <span class="text-white/70">{{ fmt(wikiSpecs.resist_distortion, 2) }}</span></div>
            <div v-if="wikiSpecs.resist_biochemical"><span class="text-white/40">Bio</span> <span class="text-white/70">{{ fmt(wikiSpecs.resist_biochemical, 2) }}</span></div>
          </div>
        </div>

        <!-- Thermal -->
        <div v-if="wikiSpecs.max_temp || wikiSpecs.overheat_temp || wikiSpecs.cooling_rate_max" class="space-y-1">
          <div class="text-white/30 text-[10px] uppercase tracking-widest">Thermal</div>
          <div class="grid grid-cols-3 gap-x-4 gap-y-1 text-xs">
            <div v-if="wikiSpecs.max_temp"><span class="text-white/40">Max</span> <span class="text-white/70">{{ fmt(wikiSpecs.max_temp, 0) }}°</span></div>
            <div v-if="wikiSpecs.overheat_temp"><span class="text-white/40">Overheat</span> <span class="text-white/70">{{ fmt(wikiSpecs.overheat_temp, 0) }}°</span></div>
            <div v-if="wikiSpecs.cooling_rate_max"><span class="text-white/40">Cooling</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.cooling_rate_max) }}/s</span></div>
            <div v-if="wikiSpecs.misfire_min_temp"><span class="text-white/40">Misfire min</span> <span class="text-white/70">{{ fmt(wikiSpecs.misfire_min_temp, 0) }}°</span></div>
            <div v-if="wikiSpecs.misfire_max_temp"><span class="text-white/40">Misfire max</span> <span class="text-white/70">{{ fmt(wikiSpecs.misfire_max_temp, 0) }}°</span></div>
            <div v-if="wikiSpecs.ir_emission"><span class="text-white/40">IR</span> <span class="text-white/70">{{ fmt(wikiSpecs.ir_emission) }}</span></div>
          </div>
        </div>

        <!-- Power -->
        <div v-if="wikiSpecs.power_draw || wikiSpecs.power_output || wikiSpecs.em_max" class="space-y-1">
          <div class="text-white/30 text-[10px] uppercase tracking-widest">Power / EM</div>
          <div class="grid grid-cols-3 gap-x-4 gap-y-1 text-xs">
            <div v-if="wikiSpecs.power_output"><span class="text-white/40">Output</span> <span class="text-amber-400">{{ fmtBig(wikiSpecs.power_output) }}</span></div>
            <div v-if="wikiSpecs.power_draw"><span class="text-white/40">Draw</span> <span class="text-white/70">{{ fmt(wikiSpecs.power_draw) }}</span></div>
            <div v-if="wikiSpecs.power_to_em"><span class="text-white/40">Pwr→EM</span> <span class="text-white/70">{{ fmt(wikiSpecs.power_to_em, 3) }}</span></div>
            <div v-if="wikiSpecs.em_min"><span class="text-white/40">EM min</span> <span class="text-white/70">{{ fmt(wikiSpecs.em_min) }}</span></div>
            <div v-if="wikiSpecs.em_max"><span class="text-white/40">EM max</span> <span class="text-white/70">{{ fmt(wikiSpecs.em_max) }}</span></div>
            <div v-if="wikiSpecs.em_decay"><span class="text-white/40">EM decay</span> <span class="text-white/70">{{ fmt(wikiSpecs.em_decay) }}</span></div>
          </div>
        </div>

        <!-- Weapon fire modes -->
        <div v-if="wikiSpecs.fire_modes.length > 0" class="space-y-1">
          <div class="text-white/30 text-[10px] uppercase tracking-widest">Fire Modes</div>
          <div v-for="(fm, i) in wikiSpecs.fire_modes" :key="i" class="flex items-center gap-3 text-xs text-white/60">
            <span class="text-white/70 font-medium">{{ fm.mode ?? fm.fire_type ?? 'Unknown' }}</span>
            <span v-if="fm.rounds_per_minute">{{ fmt(fm.rounds_per_minute, 0) }} RPM</span>
            <span v-if="fm.damage_per_second">{{ fmt(fm.damage_per_second, 1) }} DPS</span>
          </div>
        </div>

        <!-- Vehicle speed/agility/quantum -->
        <div v-if="wikiSpecs.scm_speed || wikiSpecs.max_speed" class="space-y-1">
          <div class="text-white/30 text-[10px] uppercase tracking-widest">Speed</div>
          <div class="grid grid-cols-3 gap-x-4 gap-y-1 text-xs">
            <div v-if="wikiSpecs.scm_speed"><span class="text-white/40">SCM</span> <span class="text-white/70">{{ fmt(wikiSpecs.scm_speed, 0) }} m/s</span></div>
            <div v-if="wikiSpecs.max_speed"><span class="text-white/40">Max</span> <span class="text-white/70">{{ fmt(wikiSpecs.max_speed, 0) }} m/s</span></div>
            <div v-if="wikiSpecs.boost_forward"><span class="text-white/40">Boost fwd</span> <span class="text-white/70">{{ fmt(wikiSpecs.boost_forward, 0) }} m/s</span></div>
            <div v-if="wikiSpecs.boost_backward"><span class="text-white/40">Boost rev</span> <span class="text-white/70">{{ fmt(wikiSpecs.boost_backward, 0) }} m/s</span></div>
            <div v-if="wikiSpecs.zero_to_scm"><span class="text-white/40">0→SCM</span> <span class="text-white/70">{{ fmt(wikiSpecs.zero_to_scm) }}s</span></div>
            <div v-if="wikiSpecs.zero_to_max"><span class="text-white/40">0→Max</span> <span class="text-white/70">{{ fmt(wikiSpecs.zero_to_max) }}s</span></div>
          </div>
        </div>

        <div v-if="wikiSpecs.pitch || wikiSpecs.yaw" class="space-y-1">
          <div class="text-white/30 text-[10px] uppercase tracking-widest">Agility</div>
          <div class="grid grid-cols-3 gap-x-4 gap-y-1 text-xs">
            <div v-if="wikiSpecs.pitch"><span class="text-white/40">Pitch</span> <span class="text-white/70">{{ fmt(wikiSpecs.pitch, 1) }}°/s</span></div>
            <div v-if="wikiSpecs.yaw"><span class="text-white/40">Yaw</span> <span class="text-white/70">{{ fmt(wikiSpecs.yaw, 1) }}°/s</span></div>
            <div v-if="wikiSpecs.roll"><span class="text-white/40">Roll</span> <span class="text-white/70">{{ fmt(wikiSpecs.roll, 1) }}°/s</span></div>
            <div v-if="wikiSpecs.accel_main"><span class="text-white/40">Accel</span> <span class="text-white/70">{{ fmt(wikiSpecs.accel_main, 1) }} m/s²</span></div>
            <div v-if="wikiSpecs.accel_retro"><span class="text-white/40">Retro</span> <span class="text-white/70">{{ fmt(wikiSpecs.accel_retro, 1) }} m/s²</span></div>
          </div>
        </div>

        <!-- Vehicle armor -->
        <div v-if="wikiSpecs.armor_health || wikiSpecs.armor_dmg_physical" class="space-y-1">
          <div class="text-white/30 text-[10px] uppercase tracking-widest">Armor</div>
          <div class="grid grid-cols-3 gap-x-4 gap-y-1 text-xs">
            <div v-if="wikiSpecs.armor_health"><span class="text-white/40">HP</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.armor_health) }}</span></div>
            <div v-if="wikiSpecs.armor_dmg_physical"><span class="text-white/40">Phys ×</span> <span class="text-white/70">{{ fmt(wikiSpecs.armor_dmg_physical, 2) }}</span></div>
            <div v-if="wikiSpecs.armor_dmg_energy"><span class="text-white/40">Energy ×</span> <span class="text-white/70">{{ fmt(wikiSpecs.armor_dmg_energy, 2) }}</span></div>
            <div v-if="wikiSpecs.armor_dmg_distortion"><span class="text-white/40">Dist ×</span> <span class="text-white/70">{{ fmt(wikiSpecs.armor_dmg_distortion, 2) }}</span></div>
            <div v-if="wikiSpecs.armor_dmg_thermal"><span class="text-white/40">Therm ×</span> <span class="text-white/70">{{ fmt(wikiSpecs.armor_dmg_thermal, 2) }}</span></div>
          </div>
        </div>

        <!-- Vehicle misc -->
        <div v-if="wikiSpecs.msrp || wikiSpecs.insurance_claim_time || wikiSpecs.fuel_capacity" class="space-y-1">
          <div class="text-white/30 text-[10px] uppercase tracking-widest">Misc</div>
          <div class="grid grid-cols-3 gap-x-4 gap-y-1 text-xs">
            <div v-if="wikiSpecs.msrp"><span class="text-white/40">MSRP</span> <span class="text-green-400">{{ fmtBig(wikiSpecs.msrp) }} aUEC</span></div>
            <div v-if="wikiSpecs.fuel_capacity"><span class="text-white/40">Fuel</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.fuel_capacity) }}</span></div>
            <div v-if="wikiSpecs.insurance_claim_time"><span class="text-white/40">Claim</span> <span class="text-white/70">{{ fmt(wikiSpecs.insurance_claim_time, 0) }}s</span></div>
            <div v-if="wikiSpecs.insurance_expedite_time"><span class="text-white/40">Expedite</span> <span class="text-white/70">{{ fmt(wikiSpecs.insurance_expedite_time, 0) }}s</span></div>
            <div v-if="wikiSpecs.vehicle_inventory"><span class="text-white/40">Inventory</span> <span class="text-white/70">{{ fmt(wikiSpecs.vehicle_inventory, 0) }} µSCU</span></div>
            <div v-if="wikiSpecs.pledge_url">
              <button @click="openUrl(wikiSpecs.pledge_url!)" class="text-blue-400/60 hover:text-blue-400 transition-colors cursor-pointer">Pledge ↗</button>
            </div>
          </div>
        </div>

        <!-- Dimensions -->
        <div v-if="wikiSpecs.length || wikiSpecs.width || wikiSpecs.height" class="space-y-1">
          <div class="text-white/30 text-[10px] uppercase tracking-widest">Dimensions</div>
          <div class="grid grid-cols-3 gap-x-4 gap-y-1 text-xs">
            <div v-if="wikiSpecs.length"><span class="text-white/40">Length</span> <span class="text-white/70">{{ fmt(wikiSpecs.length, 1) }}m</span></div>
            <div v-if="wikiSpecs.width"><span class="text-white/40">Width</span> <span class="text-white/70">{{ fmt(wikiSpecs.width, 1) }}m</span></div>
            <div v-if="wikiSpecs.height"><span class="text-white/40">Height</span> <span class="text-white/70">{{ fmt(wikiSpecs.height, 1) }}m</span></div>
            <div v-if="wikiSpecs.mass"><span class="text-white/40">Mass</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.mass) }} kg</span></div>
          </div>
        </div>

        <!-- Distortion -->
        <div v-if="wikiSpecs.distortion_max" class="space-y-1">
          <div class="text-white/30 text-[10px] uppercase tracking-widest">Distortion</div>
          <div class="grid grid-cols-3 gap-x-4 gap-y-1 text-xs">
            <div v-if="wikiSpecs.distortion_max"><span class="text-white/40">Max</span> <span class="text-white/70">{{ fmtBig(wikiSpecs.distortion_max) }}</span></div>
            <div v-if="wikiSpecs.distortion_decay_rate"><span class="text-white/40">Decay</span> <span class="text-white/70">{{ fmt(wikiSpecs.distortion_decay_rate) }}/s</span></div>
            <div v-if="wikiSpecs.distortion_shutdown_time"><span class="text-white/40">Shutdown</span> <span class="text-white/70">{{ fmt(wikiSpecs.distortion_shutdown_time) }}s</span></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
