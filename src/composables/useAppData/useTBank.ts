import { ref } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import type { InstrumentCard } from "../../types";

/**
 * Composable for working with the T-Bank Invest API.
 * Allows saving the token and searching instruments by ticker/name.
 */
export function useTBank() {
    const searching = ref(false);
    const searchError = ref<string | null>(null);

    const saveToken = async (token: string): Promise<void> => {
        try {
            await invoke('set_tbank_token', { token });
        } catch (e) {
            console.error("Error saving T-Bank token:", e);
        }
    };

    const searchInstruments = async (query: string): Promise<InstrumentCard[]> => {
        if (!query.trim()) return [];
        searching.value = true;
        searchError.value = null;
        try {
            return await invoke<InstrumentCard[]>('search_instruments', { query: query.trim() });
        } catch (e) {
            console.error("Error searching instruments:", e);
            searchError.value = String(e);
            return [];
        } finally {
            searching.value = false;
        }
    };

    return {
        searching,
        searchError,
        saveToken,
        searchInstruments
    };
}