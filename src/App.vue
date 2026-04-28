<script setup lang="ts">
    import {ref, onMounted, nextTick} from "vue";
    import {invoke} from "@tauri-apps/api/core";
    import CreateStrategyModal from "../src/component/createStrategyModal.vue";

    interface Strategy {
        id: number;
        name: string;
        color: string;
    }

    const strategy = ref<Strategy | null>(null);
    const allStrategies = ref<Strategy[]>([]);
    const isLoading = ref(true);
    const showModal = ref(false);

    const menuVisible = ref(false);
    const menuX = ref(0);
    const menuY = ref(0);
    const strategyIdToOp = ref<number | null>(null);

    onMounted(async () => {
        try {
            const data = await invoke<{ strategy: Strategy | null, all_strategies?: Strategy[] }>('get_data');
            strategy.value = data.strategy;
            allStrategies.value = data.all_strategies || [];
            if (allStrategies.value.length === 0) showModal.value = true;
        } catch (e) {
            console.error(e);
        } finally {
            isLoading.value = false;
        }
    });

    const openMenu = async (e: MouseEvent, id: number) => {
        e.preventDefault();
        e.stopPropagation(); // Важно: чтобы клик не ушел выше

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
    const removeStrategy = async () => {
        if (strategyIdToOp.value === null) return;
        try {
            await invoke('delete_strategy', {id: strategyIdToOp.value});

            allStrategies.value = allStrategies.value.filter(s => s.id !== strategyIdToOp.value);

            if (strategy.value?.id === strategyIdToOp.value) {
                if (allStrategies.value.length > 0) {
                    strategy.value = allStrategies.value[0];
                } else {
                    strategy.value = null;
                    showModal.value = true;
                }
            }

            menuVisible.value = false;
        } catch (e) {
            console.error("Ошибка при удалении:", e);
        }
    };
    async function handleCreate(payload: { name: string, color: string }) {
        const newStrategy = await invoke<Strategy>('create_strategy', payload);
        if (newStrategy) {
            allStrategies.value.push(newStrategy);
            strategy.value = newStrategy;
            showModal.value = false;
        }
    }
</script>

<template>
    <div class="container">
        <div v-if="isLoading">Загрузка...</div>

        <div v-else class="main-content">
            <div v-if="menuVisible"
                 class="context-menu"
                 :style="{ top: menuY + 'px', left: menuX + 'px' }"
                 @contextmenu.prevent>
                <div class="menu-item delete" @click.stop="removeStrategy">
                    Удалить стратегию
                </div>
            </div>

            <div v-if="showModal" class="modal-overlay" @click.self="showModal = false">
                <CreateStrategyModal @create="handleCreate"/>
            </div>

            <header class="header-row">
                <div class="header-left">
                    <div class="strategy-info" v-if="strategy">
                        <h1>{{ strategy.name }}</h1>
                    </div>

                    <div class="strategy-dots">
                        <div v-for="s in allStrategies" :key="s.id" class="dot-wrapper">
                            <span class="tooltip">{{ s.name }}</span>
                            <div
                                class="dot"
                                :class="{ active: strategy?.id === s.id }"
                                :style="{ backgroundColor: s.color }"
                                @click="strategy = s"
                                @contextmenu="openMenu($event, s.id)"
                            ></div>
                        </div>
                        <button class="btn-add-dot" @click="showModal = true">+</button>
                    </div>
                </div>
            </header>

            <main class="workspace">
                <div class="empty-view">
                    {{
                        strategy ? 'Рабочая область: ' + strategy.name : 'Выберите стратегию'
                    }}
                </div>
            </main>
        </div>
    </div>
</template>

<style>
    .context-menu {
        position: fixed;
        background: #1a1d21;
        border: 1px solid #444c56;
        border-radius: 8px;
        padding: 6px 0;
        z-index: 9999; /* Самый высокий приоритет */
        box-shadow: 0 10px 20px rgba(0, 0, 0, 0.6);
        min-width: 160px;
        pointer-events: all;
    }

    .menu-item {
        padding: 10px 14px;
        font-size: 13px;
        color: #94a3b8;
        cursor: pointer;
        transition: background 0.2s;
    }

    .menu-item:hover {
        background: #2d333b;
        color: white;
    }

    .menu-item.delete {
        color: #f87171;
    }

    .menu-item.delete:hover {
        background: rgba(248, 113, 113, 0.1);
    }

    html, body, #app {
        margin: 0;
        padding: 0;
        height: 100%;
        width: 100%;
        overflow: hidden;
        background: black;
        font-family: sans-serif;
    }

    .container {
        height: 100%;
        width: 100%;
        display: flex;
    }

    .main-content {
        display: flex;
        flex-direction: column;
        height: 100%;
        width: 100%;
        background: #0f1113;
    }

    .header-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0 20px;
        height: 65px;
        background: #1a1d21;
        border-bottom: 1px solid #2d333b;
    }

    .header-left {
        display: flex;
        align-items: center;
        gap: 40px;
    }

    .strategy-info h1 {
        font-size: 1.1rem;
        color: white;
        margin: 0;
        min-width: 150px;
    }

    .strategy-dots {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .dot-wrapper {
        position: relative;
        display: flex;
        flex-direction: column;
        align-items: center;
    }

    .dot {
        width: 15px;
        height: 15px;
        border-radius: 50%;
        cursor: pointer;
        border: 2px solid transparent;
        transition: all 0.2s;
    }

    .dot.active {
        border-color: white;
        box-shadow: 0 0 12px rgba(255, 255, 255, 0.2);
    }

    .tooltip {
        position: absolute;
        top: 35px;
        background: #2d333b;
        color: white;
        padding: 5px 10px;
        border-radius: 6px;
        font-size: 11px;
        opacity: 0;
        pointer-events: none;
        transition: all 0.2s;
        border: 1px solid #444c56;
        z-index: 100;
    }

    .dot-wrapper:hover .tooltip {
        opacity: 1;
        transform: translateY(5px);
    }

    .btn-add-dot {
        width: 28px;
        height: 28px;
        border-radius: 50%;
        background: transparent;
        border: 2px dashed #444c56;
        color: #444c56;
        font-size: 18px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .btn-add-dot:hover {
        border-color: #00c087;
        color: #00c087;
    }

    .modal-overlay {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.7);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 999;
        backdrop-filter: blur(4px);
    }

    .workspace {
        flex: 1;
        padding: 20px;
        color: white;
    }

    .empty-view {
        border: 2px dashed #2d333b;
        height: 100%;
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #444c56;
    }
</style>
