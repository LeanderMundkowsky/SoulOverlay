<script setup lang="ts">
import { ref, watch } from "vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import { useUserStore } from "@/stores/user";
import { useSettingsStore } from "@/stores/settings";
import { proxyImageUrl } from "@/utils/imageProxy";

const userStore = useUserStore();
const settingsStore = useSettingsStore();

const hasSecretKey = ref(false);
const canFetch = ref(false);

watch(
  () => settingsStore.settings,
  (s) => {
    hasSecretKey.value = s.uex_secret_key.length > 0;
    canFetch.value = hasSecretKey.value;
  },
  { immediate: true, deep: true },
);

watch(canFetch, (ready) => {
  if (ready && !userStore.profile && !userStore.loading) {
    userStore.loadProfile();
  }
}, { immediate: true });

function refresh() {
  if (canFetch.value) {
    userStore.loadProfile();
  }
}

const SPECIALIZATION_LABELS: Record<string, string> = {
  datarunner: "Datarunner",
  escort: "Escort",
  exploration: "Explorer",
  engineer: "Engineer",
  gunner: "Gunner",
  hauling: "Hauler",
  medical: "Medic",
  mercenary: "Mercenary",
  mining: "Miner",
  other: "Other",
  pilot: "Private Pilot",
  piracy: "Pirate",
  racer: "Racer",
  refining: "Refiner",
  refueling: "Refueler",
  repairing: "Repair",
  roleplay: "Roleplay",
  salvaging: "Salvager",
  scanning: "Scanner",
  scientist: "Scientist",
  towing: "Towing",
  trading: "Trader",
  transit: "Transit",
};

const LANGUAGE_LABELS: Record<string, string> = {
  ar: "Arabic", ca: "Catalan", zh: "Chinese", nl: "Dutch",
  en: "English", fr: "French", de: "German", it: "Italian",
  jp: "Japanese", pt: "Portuguese", ru: "Russian", es: "Spanish",
  xx: "Other",
};

const ARCHETYPE_LABELS: Record<string, string> = {
  artist: "Artist", engineer: "Engineer", explorer: "Explorer",
  lover: "Lover", novice: "Novice", outlaw: "Outlaw",
  player_one: "Player One", protector: "Protector", strategist: "Strategist",
  support: "Support", trickster: "Trickster", warlord: "Warlord",
};

const DAY_LABELS: Record<string, string> = {
  weekdays: "Weekdays", weekends: "Weekends",
};

const TIME_LABELS: Record<string, string> = {
  morning: "Morning", afternoon: "Afternoon", evening: "Evening",
};

function formatTimestamp(ts: string | null | undefined): string {
  if (!ts) return "—";
  const num = Number(ts);
  if (!isNaN(num) && num > 0) {
    return new Date(num * 1000).toLocaleDateString(undefined, {
      year: "numeric", month: "short", day: "numeric",
    });
  }
  return ts;
}
</script>

<template>
  <div class="p-6 max-w-3xl mx-auto w-full space-y-4">
    <!-- Missing keys warnings -->
    <AlertBanner
      v-if="!hasSecretKey"
      variant="warning"
      message="UEX Secret Key not configured. Set it in Settings → UEX Secret Key."
    />
    <AlertBanner
      v-if="!hasSecretKey"
      variant="warning"
      message="UEX secret key not configured. Set it in Settings → UEX Secret Key."
    />
    <AlertBanner
      v-if="userStore.error"
      variant="error"
      :message="userStore.error"
    />

    <!-- Header -->
    <div v-if="canFetch" class="flex items-center justify-between">
      <h2 class="text-white/80 text-sm font-semibold uppercase tracking-wider">
        My Profile
      </h2>
      <button
        @click="refresh"
        :disabled="userStore.loading"
        class="text-xs text-blue-400 hover:text-blue-300 disabled:text-white/20 transition-colors"
      >
        <span v-if="userStore.stale" class="text-yellow-400 mr-1">⟳</span>
        {{ userStore.loading ? "Refreshing..." : "Refresh" }}
      </button>
    </div>

    <!-- Loading -->
    <div v-if="userStore.loading && !userStore.profile" class="flex justify-center py-12">
      <LoadingSpinner />
    </div>

    <!-- Profile content -->
    <template v-if="userStore.profile">
      <!-- Identity card -->
      <div class="bg-[#1a1d24] border border-white/10 rounded-lg p-5">
        <div class="flex items-start gap-4">
          <img
            v-if="userStore.profile.avatar"
            :src="proxyImageUrl(userStore.profile.avatar)"
            :alt="userStore.profile.name"
            class="w-16 h-16 rounded-full object-cover bg-white/5 shrink-0"
            @error="($event.target as HTMLImageElement).style.display = 'none'"
          />
          <div
            v-else
            class="w-16 h-16 rounded-full bg-white/10 flex items-center justify-center shrink-0"
          >
            <span class="text-white/40 text-xl font-bold">
              {{ userStore.profile.name.charAt(0).toUpperCase() }}
            </span>
          </div>
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2 flex-wrap">
              <span class="text-white text-lg font-semibold">{{ userStore.profile.name }}</span>
              <span
                v-if="userStore.profile.is_staff"
                class="text-xs px-1.5 py-0.5 rounded bg-purple-500/20 text-purple-400"
              >Staff</span>
              <span
                v-if="userStore.profile.is_datarunner"
                class="text-xs px-1.5 py-0.5 rounded bg-teal-500/20 text-teal-400"
              >Datarunner</span>
              <span
                v-if="userStore.profile.is_away_game"
                class="text-xs px-1.5 py-0.5 rounded bg-yellow-500/20 text-yellow-400"
              >Away</span>
            </div>
            <div class="text-white/50 text-sm mt-0.5">@{{ userStore.profile.username }}</div>
            <p
              v-if="userStore.profile.bio"
              class="text-white/40 text-sm mt-2 leading-relaxed"
            >{{ userStore.profile.bio }}</p>
          </div>
        </div>
      </div>

      <!-- Contact & Links -->
      <div
        v-if="userStore.profile.email || userStore.profile.website_url || userStore.profile.discord_username || userStore.profile.twitch_username"
        class="bg-[#1a1d24] border border-white/10 rounded-lg p-5 space-y-2"
      >
        <h3 class="text-white/60 text-xs font-semibold uppercase tracking-wider mb-3">Contact & Links</h3>
        <div class="grid grid-cols-2 gap-x-6 gap-y-2 text-sm">
          <template v-if="userStore.profile.email">
            <span class="text-white/40">Email</span>
            <span class="text-white/80 truncate">{{ userStore.profile.email }}</span>
          </template>
          <template v-if="userStore.profile.website_url">
            <span class="text-white/40">Website</span>
            <span class="text-blue-400/80 truncate">{{ userStore.profile.website_url }}</span>
          </template>
          <template v-if="userStore.profile.discord_username">
            <span class="text-white/40">Discord</span>
            <span class="text-white/80">{{ userStore.profile.discord_username }}</span>
          </template>
          <template v-if="userStore.profile.twitch_username">
            <span class="text-white/40">Twitch</span>
            <span class="text-purple-400/80">{{ userStore.profile.twitch_username }}</span>
          </template>
          <template v-if="userStore.profile.timezone">
            <span class="text-white/40">Timezone</span>
            <span class="text-white/80">{{ userStore.profile.timezone }}</span>
          </template>
          <template v-if="userStore.profile.language">
            <span class="text-white/40">Language</span>
            <span class="text-white/80">{{ userStore.profile.language }}</span>
          </template>
        </div>
      </div>

      <!-- Availability -->
      <div
        v-if="userStore.profile.day_availability.length > 0 || userStore.profile.time_availability.length > 0"
        class="bg-[#1a1d24] border border-white/10 rounded-lg p-5"
      >
        <h3 class="text-white/60 text-xs font-semibold uppercase tracking-wider mb-3">Availability</h3>
        <div class="flex flex-wrap gap-2">
          <span
            v-for="d in userStore.profile.day_availability"
            :key="'day-' + d"
            class="text-xs px-2 py-1 rounded bg-blue-500/15 text-blue-400"
          >{{ DAY_LABELS[d] ?? d }}</span>
          <span
            v-for="t in userStore.profile.time_availability"
            :key="'time-' + t"
            class="text-xs px-2 py-1 rounded bg-sky-500/15 text-sky-400"
          >{{ TIME_LABELS[t] ?? t }}</span>
        </div>
      </div>

      <!-- Specializations -->
      <div
        v-if="userStore.profile.specializations.length > 0"
        class="bg-[#1a1d24] border border-white/10 rounded-lg p-5"
      >
        <h3 class="text-white/60 text-xs font-semibold uppercase tracking-wider mb-3">Specializations</h3>
        <div class="flex flex-wrap gap-2">
          <span
            v-for="s in userStore.profile.specializations"
            :key="'spec-' + s"
            class="text-xs px-2 py-1 rounded bg-teal-500/15 text-teal-400"
          >{{ SPECIALIZATION_LABELS[s] ?? s }}</span>
        </div>
      </div>

      <!-- Languages -->
      <div
        v-if="userStore.profile.languages.length > 0"
        class="bg-[#1a1d24] border border-white/10 rounded-lg p-5"
      >
        <h3 class="text-white/60 text-xs font-semibold uppercase tracking-wider mb-3">Languages</h3>
        <div class="flex flex-wrap gap-2">
          <span
            v-for="l in userStore.profile.languages"
            :key="'lang-' + l"
            class="text-xs px-2 py-1 rounded bg-indigo-500/15 text-indigo-400"
          >{{ LANGUAGE_LABELS[l] ?? l }}</span>
        </div>
      </div>

      <!-- Archetypes -->
      <div
        v-if="userStore.profile.archetypes.length > 0"
        class="bg-[#1a1d24] border border-white/10 rounded-lg p-5"
      >
        <h3 class="text-white/60 text-xs font-semibold uppercase tracking-wider mb-3">Archetypes</h3>
        <div class="flex flex-wrap gap-2">
          <span
            v-for="a in userStore.profile.archetypes"
            :key="'arch-' + a"
            class="text-xs px-2 py-1 rounded bg-amber-500/15 text-amber-400"
          >{{ ARCHETYPE_LABELS[a] ?? a }}</span>
        </div>
      </div>

      <!-- Verification & Dates -->
      <div class="bg-[#1a1d24] border border-white/10 rounded-lg p-5">
        <h3 class="text-white/60 text-xs font-semibold uppercase tracking-wider mb-3">Status & Dates</h3>
        <div class="grid grid-cols-2 gap-x-6 gap-y-2 text-sm">
          <span class="text-white/40">RSI Verified</span>
          <span :class="userStore.profile.date_rsi_verified ? 'text-green-400' : 'text-white/30'">
            {{ userStore.profile.date_rsi_verified ? formatTimestamp(userStore.profile.date_rsi_verified) : 'Not verified' }}
          </span>
          <span class="text-white/40">Twitch Verified</span>
          <span :class="userStore.profile.date_twitch_verified ? 'text-green-400' : 'text-white/30'">
            {{ userStore.profile.date_twitch_verified ? formatTimestamp(userStore.profile.date_twitch_verified) : 'Not verified' }}
          </span>
          <span class="text-white/40">Member Since</span>
          <span class="text-white/80">{{ formatTimestamp(userStore.profile.date_added) }}</span>
          <span class="text-white/40">Last Updated</span>
          <span class="text-white/80">{{ formatTimestamp(userStore.profile.date_modified) }}</span>
        </div>
      </div>
    </template>

    <!-- Empty state -->
    <div
      v-if="canFetch && !userStore.loading && !userStore.profile && !userStore.error"
      class="text-center text-white/30 py-12 text-sm"
    >
      No profile data available.
    </div>
  </div>
</template>
