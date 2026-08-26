import { ref, type Ref } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import type { Asset, Source } from "../../types";

// FIX: Pass references to allAssets and the loadData function as dependencies
export function useTransactions(
    _allSources: Ref<Source[]>, // Warning fix: prefixed with _
    allAssets: Ref<Asset[]>,
    loadData: () => Promise<void>
) {
    const showTxModal = ref(false);
    const txMode = ref<'deposit' | 'withdraw'>('deposit');
    const activeSourceForTx = ref<Source | null>(null);

    const openTransactionModal = (source: Source, mode: 'deposit' | 'withdraw') => {
        activeSourceForTx.value = source;
        txMode.value = mode;
        showTxModal.value = true;
    };

    const handleTransactionSubmit = async (payload: { amount: number; timestamp: string }) => {
        if (!activeSourceForTx.value) return;

        try {
            console.log("[DEBUG] Running transaction for wallet:", txMode.value);

            // 1. If it's a regular deposit, run our investment pipeline
            if (txMode.value === 'deposit') {
                // Invoke the share-distribution command which returns Vec<Asset>
                const updatedAssetsList = await invoke<Asset[]>('execute_source_deposit', {
                    source_id: Number(activeSourceForTx.value.id),
                    total_amount: Number(payload.amount),
                    currency: "RUB" // Default base currency
                });

                // INSTANT BUBBLE INFLATION: allAssets is now available and reactively inflates RUB bubbles
                allAssets.value = [...updatedAssetsList];
                console.log("[DEBUG] RUB bubbles created and distributed across groups!");
            } else if (txMode.value === 'withdraw') {
                // Withdrawal (debit) logic
            }

            // 2. Reload all data globally to update the text wallet balances in the UI
            await loadData();

        } catch (e) {
            console.error("Error processing source transaction:", e);
        } finally {
            // CRITICAL FIX: Replaced the erroneous 'Implemented' with a valid 'finally' block
            showTxModal.value = false;
        }
    };

    return {
        showTxModal,
        txMode,
        activeSourceForTx,
        openTransactionModal,
        handleTransactionSubmit
    };
}
