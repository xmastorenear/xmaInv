import { ref, computed, nextTick, type Ref } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import type { Source, Strategy } from "../../types";

export function useSources(activeStrategy: Ref<Strategy | null>) {
    const allSources = ref<Source[]>([]);
    const showSourceModal = ref(false);
    const showSourceRenameModal = ref(false);
    const newSourceName = ref('');

    const menuVisible = ref(false);
    const menuX = ref(0);
    const menuY = ref(0);
    const sourceIdToOp = ref<number | null>(null);

    const filteredSources = computed(() => {
        if (!activeStrategy.value) return [];
        return allSources.value.filter(s => s.strategy_id === activeStrategy.value!.id);
    });

    const openSourceMenu = async (e: MouseEvent, id: number) => {
        e.preventDefault();
        e.stopPropagation();
        sourceIdToOp.value = id;
        menuX.value = e.clientX;
        menuY.value = e.clientY;
        menuVisible.value = true;

        await nextTick();
        const close = () => {
            menuVisible.value = false;
            window.removeEventListener('click', close);
        };
        window.addEventListener('click', close);
    };

    const handleCreateSource = async (payload: { name: string }) => {
        if (!activeStrategy.value) return;
        try {
            const res = await invoke<Source>('create_source', {
                strategy_id: activeStrategy.value.id,
                name: payload.name,
                icon_url: ""
            });
            allSources.value.push(res);
            showSourceModal.value = false;
        } catch (e) { console.error(e); }
    };

    const handleDeleteSource = async () => {
        if (sourceIdToOp.value === null) return;
        try {
            await invoke('delete_source', { id: sourceIdToOp.value });
            allSources.value = allSources.value.filter(s => s.id !== sourceIdToOp.value);
            menuVisible.value = false;
        } catch (e) { console.error(e); }
    };

    const openSourceRename = () => {
        const target = allSources.value.find(s => s.id === sourceIdToOp.value);
        if (target) {
            newSourceName.value = target.name;
            showSourceRenameModal.value = true;
        }
        menuVisible.value = false;
    };

    const handleSourceRename = async () => {
        if (sourceIdToOp.value === null || !newSourceName.value.trim()) return;
        try {
            await invoke('rename_source', { id: sourceIdToOp.value, new_name: newSourceName.value.trim() });
            const src = allSources.value.find(s => s.id === sourceIdToOp.value);
            if (src) src.name = newSourceName.value.trim();
            showSourceRenameModal.value = false;
        } catch (e) { console.error(e); }
    };

    return {
        allSources, filteredSources, showSourceModal, showSourceRenameModal,
        newSourceName, menuVisible, menuX, menuY, sourceIdToOp,
        openSourceMenu, handleCreateSource, handleDeleteSource, openSourceRename, handleSourceRename
    };
}
