import { ref, type Ref } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import type { Source } from "../../types";

export function useTransactions(allSources: Ref<Source[]>) {
    const showTxModal = ref(false);
    const txMode = ref<'deposit' | 'withdraw'>('deposit');
    const activeSourceForTx = ref<Source | null>(null);
    const openTransactionModal = (source: Source, mode: 'deposit' | 'withdraw') => {
        activeSourceForTx.value = source;
        txMode.value = mode;
        showTxModal.value = true;
    };
    const handleTransactionSubmit = async (payload: { amount: number, timestamp: string }) => {
        if (!activeSourceForTx.value) return;
        try {
            await invoke('add_transaction', {
                source_id: activeSourceForTx.value.id,
                amount: payload.amount,
                timestamp: payload.timestamp,
                description: payload.amount > 0 ? "Deposit" : "Withdraw"
            });

            const src = allSources.value.find(s => s.id === activeSourceForTx.value!.id);
            if (src) src.total_balance += payload.amount;

            showTxModal.value = false;
        } catch (e) { console.error(e); }
    };

    return { showTxModal, txMode, activeSourceForTx, openTransactionModal, handleTransactionSubmit };
}
