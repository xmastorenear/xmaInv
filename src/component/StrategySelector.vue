<script setup lang="ts">
    import { ref, nextTick } from "vue";
    import { invoke } from "@tauri-apps/api/core";

    interface Strategy {
        id: number;
        name: string;
        color: string;
    }

    const props = defineProps<{
        allStrategies: Strategy[];
        activeStrategy: Strategy | null;
    }>();

    const emit = defineEmits<{
        (e: 'select', s: Strategy): void;
        (e: 'delete', id: number): void;
        (e: 'open-create'): void;
        (e: 'rename', id: number, newName: string): void;
        (e: 'refresh'): void;
    }>();

    const menuVisible = ref(false);
    const menuX = ref(0);
    const menuY = ref(0);
    const strategyIdToOp = ref<number | null>(null);

    const showRenameModal = ref(false);
    const newName = ref('');
    const openMenu = async (e: MouseEvent, id: number) => {
        e.preventDefault();
        e.stopPropagation();

        strategyIdToOp.value = id;
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

    const handleDelete = () => {
        if (strategyIdToOp.value !== null) {
            emit('delete', strategyIdToOp.value);
            menuVisible.value = false;
        }
    };

    const openRename = () => {
        const target = props.allStrategies.find(s => s.id === strategyIdToOp.value);
        if (target) {
            newName.value = target.name;
            showRenameModal.value = true;
        }
        menuVisible.value = false;
    };

    const handleRename = async () => {
        debugger;
        if (strategyIdToOp.value !== null && newName.value.trim()) {
            emit('rename', strategyIdToOp.value, newName.value.trim());
            showRenameModal.value = false;
        }

        try {
            await invoke('rename_strategy', {
                id: strategyIdToOp.value,
                new_name: newName.value.trim()
            });

            const s = props.allStrategies.find(item => item.id === strategyIdToOp.value);
            if (s) s.name = newName.value.trim();

            if (props.activeStrategy?.id === strategyIdToOp.value) {
                emit('refresh');
            }

            showRenameModal.value = false;
        } catch (e) {
            console.error("Rename error:", e);
        }
    };
</script>

<template>
    <div class="strategy-dots">
        <Teleport to="body">
            <Transition name="fade-scale">
                <div v-if="menuVisible"
                     class="context-menu"
                     :style="{ top: menuY + 'px', left: menuX + 'px' }"
                     @contextmenu.prevent>

                    <div class="menu-item" @click="openRename">
                        <i class="pi pi-pencil"></i>
                        <span>{{ $t('common.rename') }}</span>
                    </div>

                    <div class="menu-divider"></div>

                    <div class="menu-item delete" @click.stop="handleDelete">
                        <i class="pi pi-trash"></i>
                        <span>{{ $t('common.delete') }}</span>
                    </div>
                </div>
            </Transition>

            <Transition name="modal-fade">
                <div v-if="showRenameModal" class="modal-overlay" @click.self="showRenameModal = false">
                    <div class="modal-card">
                        <h2>{{ $t('common.rename') }}</h2>
                        <input
                            v-model="newName"
                            @keyup.enter="handleRename"
                            autofocus
                            class="rename-input"
                            :placeholder="$t('strategy.examplePlaceHolder')"
                        >
                        <div class="modal-actions">
                            <button @click="showRenameModal = false" class="btn-cancel">{{ $t('common.cancel') }}</button>
                            <button @click="handleRename" class="btn-submit deposit" :disabled="!newName.trim()">
                                {{ $t('common.save') }}
                            </button>
                        </div>
                    </div>
                </div>
            </Transition>
        </Teleport>

        <span class="selector-label">{{ $t('strategy.title') }}</span>

        <div v-for="s in allStrategies" :key="s.id" class="dot-wrapper">
            <span class="tooltip">{{ s.name }}</span>
            <div class="dot"
                 :class="{ active: activeStrategy?.id === s.id }"
                 :style="{ backgroundColor: s.color }"
                 @click="emit('select', s)"
                 @contextmenu="openMenu($event, s.id)">
            </div>
        </div>

        <button class="btn-add-dot" @click="emit('open-create')" title="Добавить">+</button>
    </div>
</template>

<style scoped>
    .strategy-dots {
        display: flex;
        align-items: center;
        gap: 12px;
        background: rgba(255, 255, 255, 0.03);
        padding: 6px 12px;
        border-radius: 20px;
        border: 1px solid rgba(255, 255, 255, 0.05);
    }

    .modal-overlay {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.75);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 10001;
        backdrop-filter: blur(4px);
    }

    .selector-label {
        font-size: 0.65rem;
        color: #444c56;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        font-weight: 700;
        margin-right: 4px;
        user-select: none;
    }

    .modal-card {
        background: #1a1d21;
        padding: 24px;
        border-radius: 16px;
        border: 1px solid #2d333b;
        width: 100%;
        max-width: 360px;
        text-align: center;
        box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
    }

    .modal-card h2 {
        color: white;
        margin: 0 0 16px 0;
        font-size: 1.25rem;
    }

    .dot-wrapper { position: relative; display: flex; flex-direction: column; align-items: center; }

    .dot {
        width: 14px;
        height: 14px;
        border-radius: 50%;
        cursor: pointer;
        border: 2px solid transparent;
        transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        opacity: 0.6;
    }

    .dot.active {
        opacity: 1;
        border-color: white;
        transform: scale(1.3);
        box-shadow: 0 0 10px v-bind('activeStrategy?.color || "transparent"');
    }

    .dot:hover { opacity: 1; transform: scale(1.2); }

    .tooltip {
        position: absolute; top: 38px; background: #2d333b; color: white;
        padding: 6px 12px; border-radius: 6px; font-size: 11px;
        opacity: 0; pointer-events: none; transition: 0.2s ease;
        border: 1px solid #444c56; z-index: 100; white-space: nowrap;
    }

    .dot-wrapper:hover .tooltip { opacity: 1; transform: translateY(4px); }

    .btn-add-dot {
        width: 24px; height: 24px; border: 1px dashed #444c56;
        background: transparent; color: #444c56; border-radius: 50%;
        display: flex; align-items: center; justify-content: center;
        cursor: pointer; transition: 0.2s;
    }

    .btn-add-dot:hover { color: #00c087; border-color: #00c087; background: rgba(0, 192, 135, 0.05); }

    .context-menu {
        position: fixed; background: #1a1d21;
        border: 1px solid #444c56; border-radius: 10px;
        padding: 6px; z-index: 10000;
        box-shadow: 0 12px 24px rgba(0, 0, 0, 0.5); min-width: 170px;
    }

    .menu-item {
        display: flex; align-items: center; gap: 10px;
        padding: 10px 14px; font-size: 13px; color: #94a3b8;
        cursor: pointer; border-radius: 6px; transition: 0.2s;
    }
    .menu-item:hover { background: #2d333b; color: white; }

    .menu-item.delete { color: #f87171; }

    .menu-item.delete:hover { background: rgba(248, 113, 113, 0.1); }

    .menu-divider { height: 1px; background: #2d333b; margin: 4px 8px; }

    .rename-input {
        width: 93%; padding: 12px; background: #0f1113;
        border: 1px solid #2d333b; border-radius: 8px;
        color: white; margin: 20px 0; font-size: 1rem; outline: none;
    }
    .rename-input:focus { border-color: #00c087; }

    .modal-actions { display: flex; gap: 12px; }

    .btn-submit { background: #00c087; color: black; border: none; flex: 1; padding: 12px; border-radius: 8px; font-weight: bold; cursor: pointer; }

    .btn-cancel { background: transparent; border: 1px solid #2d333b; color: #94a3b8; flex: 1; padding: 12px; border-radius: 8px; cursor: pointer; }

    .fade-scale-enter-active, .fade-scale-leave-active { transition: all 0.15s ease; }

    .fade-scale-enter-from, .fade-scale-leave-to { opacity: 0; transform: scale(0.95) translateY(-5px); }
</style>
