<script setup lang="ts">
    import { ref, nextTick } from "vue";

    interface Strategy {
        id: number;
        name: string;
        color: string;
    }

    defineProps<{
        allStrategies: Strategy[];
        activeStrategy: Strategy | null;
    }>();

    const emit = defineEmits<{
        (e: 'select', s: Strategy): void;
        (e: 'delete', id: number): void;
        (e: 'open-create'): void;
    }>();

    const menuVisible = ref(false);
    const menuX = ref(0);
    const menuY = ref(0);
    const strategyIdToOp = ref<number | null>(null);

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
</script>

<template>
    <div class="strategy-dots">
        <!-- Контекстное меню выносим в Teleport, чтобы оно не обрезалось родителем -->
        <Teleport to="body">
            <Transition name="fade-scale">
                <div v-if="menuVisible"
                     class="context-menu"
                     :style="{ top: menuY + 'px', left: menuX + 'px' }"
                     @contextmenu.prevent>
                    <div class="menu-item delete" @click.stop="handleDelete">
                        <svg xmlns="http://w3.org" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2M10 11v6M14 11v6"/></svg>
                        <span>Удалить стратегию</span>
                    </div>
                </div>
            </Transition>
        </Teleport>

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

    .strategy-dots { display: flex; align-items: center; gap: 14px; }
    .dot-wrapper { position: relative; display: flex; flex-direction: column; align-items: center; }
    .dot {
        width: 16px; height: 16px;
        border-radius: 50%; cursor: pointer;
        border: 2px solid transparent; transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
    }
    .dot.active { border-color: white; box-shadow: 0 0 15px rgba(255, 255, 255, 0.3); transform: scale(1.1); }
    .dot:hover { transform: scale(1.25); }

    .tooltip {
        position: absolute; top: 38px; background: #2d333b; color: white;
        padding: 6px 12px; border-radius: 6px; font-size: 11px;
        opacity: 0; pointer-events: none; transition: 0.2s ease;
        border: 1px solid #444c56; z-index: 100; white-space: nowrap;
    }
    .dot-wrapper:hover .tooltip { opacity: 1; transform: translateY(4px); }

    .btn-add-dot {
        width: 28px; height: 28px;
        border-radius: 50%; background: #1a1d21;
        border: 1px dashed #444c56; color: #94a3b8;
        font-size: 18px; cursor: pointer;
        display: flex; align-items: center; justify-content: center;
        transition: all 0.3s ease;
    }
    .btn-add-dot:hover {
        border-color: #00c087; color: #00c087;
        transform: rotate(90deg); border-style: solid;
        box-shadow: 0 0 10px rgba(0, 192, 135, 0.2);
    }

    .context-menu {
        position: fixed; background: #1a1d21;
        border: 1px solid #444c56; border-radius: 10px;
        padding: 6px; z-index: 10000;
        box-shadow: 0 12px 24px rgba(0, 0, 0, 0.5); min-width: 170px;
    }
    .menu-item.delete {
        display: flex; align-items: center; gap: 10px;
        padding: 10px 14px; font-size: 13px; color: #fdfdfd;
        cursor: pointer; border-radius: 6px; transition: 0.2s;
    }
    .menu-item.delete:hover { background: rgba(248, 113, 113, 0.1); padding-left: 18px; }

    .fade-scale-enter-active, .fade-scale-leave-active { transition: all 0.15s ease; }
    .fade-scale-enter-from, .fade-scale-leave-to { opacity: 0; transform: scale(0.95) translateY(-5px); }
</style>
