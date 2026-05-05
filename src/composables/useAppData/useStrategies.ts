import { ref } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import type { Strategy } from "../../types";

export function useStrategies() {
    const strategy = ref<Strategy | null>(null);
    const allStrategies = ref<Strategy[]>([]);
    const showModal = ref(false);

    const handleCreateStrategy = async (payload: { name: string, color: string }) => {
        try {
            const res = await invoke<Strategy>('create_strategy', payload);
            allStrategies.value.push(res);
            strategy.value = res;
            showModal.value = false;
        } catch (e) { console.error(e); }
    };

    const handleDeleteStrategy = async (id: number) => {
        try {
            await invoke('delete_strategy', { id });
            allStrategies.value = allStrategies.value.filter(s => s.id !== id);
            if (strategy.value?.id === id) {
                strategy.value = allStrategies.value[0] || null;
                if (!strategy.value) showModal.value = true;
            }
        } catch (e) { console.error(e); }
    };

    const handleRenameStrategy = async (id: number, newName: string) => {
        try {
            await invoke('rename_strategy', { id, new_name: newName });
            const target = allStrategies.value.find(s => s.id === id);
            if (target) target.name = newName;
            if (strategy.value?.id === id) strategy.value.name = newName;
        } catch (e) { console.error(e); }
    };

    return {
        strategy, allStrategies, showModal,
        handleCreateStrategy, handleDeleteStrategy, handleRenameStrategy
    };
}
