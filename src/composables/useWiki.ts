import { ref } from "vue";
import { commands } from "@/bindings";
import type { UexResult, WikiEntitySpecs } from "@/bindings";

/**
 * Composable for Star Citizen Wiki API interactions.
 * Handles supplemental search and entity spec enrichment.
 */
export function useWiki() {
  const wikiResults = ref<UexResult[]>([]);
  const wikiLoading = ref(false);

  const wikiSpecs = ref<WikiEntitySpecs | null>(null);
  const wikiSpecsLoading = ref(false);
  const wikiSpecsError = ref<string | null>(null);

  /**
   * Search the Wiki for items and vehicles by name.
   * Returns results with source="wiki".
   */
  async function searchWiki(query: string): Promise<UexResult[]> {
    if (!query.trim()) {
      wikiResults.value = [];
      return [];
    }

    wikiLoading.value = true;
    try {
      const result = await commands.wikiSearch(query);
      if (result.status === "ok" && result.data.ok && result.data.data) {
        wikiResults.value = result.data.data;
        return result.data.data;
      }
      wikiResults.value = [];
      return [];
    } catch (e) {
      console.warn("Wiki search failed:", e);
      wikiResults.value = [];
      return [];
    } finally {
      wikiLoading.value = false;
    }
  }

  /**
   * Fetch detailed Wiki specs for an entity.
   * Uses 7-day cache on the backend.
   */
  async function getWikiSpecs(kind: string, entityId: string, entityName: string, uuid: string) {
    wikiSpecsLoading.value = true;
    wikiSpecsError.value = null;
    wikiSpecs.value = null;

    try {
      const result = await commands.wikiEntitySpecs(kind, entityId, entityName, uuid);
      if (result.status === "ok" && result.data.ok && result.data.data) {
        wikiSpecs.value = result.data.data;
      } else {
        const errMsg = result.status === "ok" ? result.data.error : String(result.error);
        wikiSpecsError.value = errMsg ?? "No wiki data available";
      }
    } catch (e) {
      wikiSpecsError.value = String(e);
    } finally {
      wikiSpecsLoading.value = false;
    }
  }

  return {
    wikiResults,
    wikiLoading,
    wikiSpecs,
    wikiSpecsLoading,
    wikiSpecsError,
    searchWiki,
    getWikiSpecs,
  };
}
