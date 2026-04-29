<script setup lang="ts">
    import { ref, onMounted } from "vue";
    import { invoke } from "@tauri-apps/api/core";
    import StrategySelector from "./component/StrategySelector.vue";
    import CreateStrategyModal from "./component/CreateStrategyModal.vue";

    interface Strategy {
        id: number;
        name: string;
        color: string;
    }

    const strategy = ref<Strategy | null>(null);
    const allStrategies = ref<Strategy[]>([]);
    const isLoading = ref(true);
    const showModal = ref(false);

    onMounted(async () => {
        try {
            const data = await invoke<{ strategy: Strategy | null, all_strategies?: Strategy[] }>('get_data');
            strategy.value = data.strategy;
            allStrategies.value = data.all_strategies || [];

            if (allStrategies.value.length === 0) {
                showModal.value = true;
            }
        } catch (e) {
            console.error("Failed to load initial data:", e);
        } finally {
            isLoading.value = false;
        }
    });

    async function handleCreate(payload: { name: string, color: string }) {
        try {
            const newStrategy = await invoke<Strategy>('create_strategy', payload);
            if (newStrategy) {
                allStrategies.value.push(newStrategy);
                strategy.value = newStrategy;
                showModal.value = false;
            }
        } catch (e) {
            console.error("Create error:", e);
        }
    }

    async function handleDelete(id: number) {
        try {
            await invoke('delete_strategy', { id });
            allStrategies.value = allStrategies.value.filter(s => s.id !== id);

            // Если удалили активную — переключаемся на первую оставшуюся
            if (strategy.value?.id === id) {
                if (allStrategies.value.length > 0) {
                    strategy.value = allStrategies.value[0];
                } else {
                    strategy.value = null;
                    showModal.value = true;
                }
            }
        } catch (e) {
            console.error("Delete error:", e);
        }
    }
</script>

<template>
    <div class="container">
        <div v-if="isLoading" class="loader">Инициализация...</div>

        <div v-else class="main-content">
            <Transition name="modal-fade">
                <div v-if="showModal" class="modal-overlay" @click.self="showModal = false">
                    <CreateStrategyModal @create="handleCreate"/>
                </div>
            </Transition>

            <header class="header-row">
                <div class="header-left">
                    <div class="strategy-info" v-if="strategy">
                        <h1>{{ strategy.name }}</h1>
                    </div>

                    <StrategySelector
                        :all-strategies="allStrategies"
                        :active-strategy="strategy"
                        @select="s => strategy = s"
                        @delete="handleDelete"
                        @open-create="showModal = true"
                    />
                </div>
            </header>

            <main class="workspace">
                <Transition name="fade" mode="out-in">
                    <div :key="strategy?.id || 'none'" class="empty-view">
                        <span v-if="strategy">Рабочая область: {{ strategy.name }}</span>
                        <span v-else>Выберите или создайте стратегию</span>
                    </div>
                </Transition>
            </main>
        </div>
    </div>
</template>

<style>
    html, body, #app {
        margin: 0;
        padding: 0;
        height: 100%;
        width: 100%;
        overflow: hidden;
        font-family: 'Segoe UI', system-ui, sans-serif;
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
        padding: 0 24px;
        margin: 10px;
        height: 65px;
        background: #000000;
        border: 1px solid #272727;
        border-radius: 10px;
    }

    .header-left {
        display: flex;
        align-items: center;
        gap: 40px;
    }

    .strategy-info h1 {
        font-size: 1.1rem;
        color: #fff;
        margin: 0;
        min-width: 140px;
    }

    .modal-overlay {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.75);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 999;
        backdrop-filter: blur(8px);
    }

    .workspace {
        flex: 1;
        padding: 10px;
    }

    .empty-view {
        border: 1px solid #272727;
        height: 100%;
        border-radius: 10px;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #444c56;
        font-size: 1.1rem;
        background-color: #000000;
    }


    .modal-fade-enter-active, .modal-fade-leave-active { transition: opacity 0.3s ease; }
    .modal-fade-enter-from, .modal-fade-leave-to { opacity: 0; }

    .fade-enter-active, .fade-leave-active { transition: opacity 0.2s ease; }
    .fade-enter-from, .fade-leave-to { opacity: 0; }

    .loader {
        width: 100%;
        height: 100%;
        display: flex;
        justify-content: center;
        align-items: center;
        color: #94a3b8;
        background: #000;
    }
</style>
