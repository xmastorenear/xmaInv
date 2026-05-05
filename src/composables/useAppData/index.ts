import { ref } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import type { AppDataResponse } from "../../types";
import { useStrategies } from "./useStrategies";
import { useSources } from "./useSources";
import { useTransactions } from "./useTransactions";

export function useAppData() {
    const isLoading = ref(true);

    const strategies = useStrategies();
    const sources = useSources(strategies.strategy);
    const transactions = useTransactions(sources.allSources);

    const loadData = async () => {
        try {
            const data = await invoke<AppDataResponse>('get_data');
            strategies.allStrategies.value = data.all_strategies || [];
            sources.allSources.value = data.sources || [];

            if (data.strategy) {
                strategies.strategy.value = data.strategy;
            } else if (strategies.allStrategies.value.length > 0) {
                strategies.strategy.value = strategies.allStrategies.value[0];
            } else {
                strategies.showModal.value = true;
            }
        } catch (e) {
            console.error("Load error:", e);
        } finally {
            isLoading.value = false;
        }
    };

    return {
        isLoading,
        loadData,
        ...strategies,
        ...sources,
        ...transactions
    };
}
