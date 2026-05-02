<script setup lang="ts">
    import { ref, onMounted, computed, nextTick } from "vue";
    import { invoke } from "@tauri-apps/api/core";
    import StrategySelector from "./component/StrategySelector.vue";
    import CreateStrategyModal from "./component/CreateStrategyModal.vue";
    import SourceCard from "./component/SourceCard.vue";
    import LanguageSwitcher from "./component/LanguageSwitcher.vue";
    import CreateSourceModal from "./component/CreateSourceModal.vue";
    import TransactionModal from "./component/TransactionModal.vue";

    interface Strategy { id: number; name: string; color: string; }
    interface Source {
        id: number;
        strategy_id: number;
        name: string;
        icon_url: string;
        total_balance: number;
        profit_loss: number;
    }

    // Состояния
    const strategy = ref<Strategy | null>(null);
    const allStrategies = ref<Strategy[]>([]);
    const allSources = ref<Source[]>([]);
    const isLoading = ref(true);
    const showModal = ref(false);
    const showSourceModal = ref(false);

    // Контекстное меню для источников
    const menuVisible = ref(false);
    const menuX = ref(0);
    const menuY = ref(0);
    const sourceIdToDelete = ref<number | null>(null);
    const sourceMenuVisible = ref(false);

    const showTxModal = ref(false);
    const txMode = ref<'deposit' | 'withdraw'>('deposit');
    const activeSourceForTx = ref<Source | null>(null);

    const showSourceRenameModal = ref(false);
    const newSourceName = ref('');

    // Открыть модалку переименования из контекстного меню
    const openSourceRename = () => {
        const target = allSources.value.find(s => s.id === sourceIdToDelete.value);
        if (target) {
            newSourceName.value = target.name;
            showSourceRenameModal.value = true;
        }
        menuVisible.value = false; // Закрываем контекстное меню
    };

    // Сохранить новое имя
    const handleSourceRename = async () => {
        if (sourceIdToDelete.value === null || !newSourceName.value.trim()) return;

        try {
            await invoke('rename_source', {
                id: sourceIdToDelete.value,
                new_name: newSourceName.value.trim()
            });

            // Обновляем локально
            const src = allSources.value.find(s => s.id === sourceIdToDelete.value);
            if (src) src.name = newSourceName.value.trim();

            showSourceRenameModal.value = false;
        } catch (e) {
            console.error("Rename source error:", e);
        }
    };

    const filteredSources = computed(() => {
        if (!strategy.value) return [];
        return allSources.value.filter(s => s.strategy_id === strategy.value!.id);
    });

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
                timestamp: payload.timestamp, // Используем время из модалки
                description: payload.amount > 0 ? "Deposit" : "Withdraw"
            });

            updateSourceBalance(activeSourceForTx.value.id, payload.amount);
            showTxModal.value = false;
        } catch (e) {
            console.error(e);
        }
    };



    onMounted(async () => {
        try {
            const data = await invoke<any>('get_data');
            strategy.value = data.strategy;
            allStrategies.value = data.all_strategies || [];
            allSources.value = data.sources || [];
            if (allStrategies.value.length === 0) showModal.value = true;
        } catch (e) { console.error(e); } finally { isLoading.value = false; }
    });

    // Работа со стратегиями
    const handleCreateStrategy = async (payload: { name: string, color: string }) => {
        const res = await invoke<Strategy>('create_strategy', payload);
        if (res) {
            allStrategies.value.push(res);
            strategy.value = res;
            showModal.value = false;
        }
    };



    const handleDeleteStrategy = async (id: number) => {
        await invoke('delete_strategy', { id });
        allStrategies.value = allStrategies.value.filter(s => s.id !== id);
        allSources.value = allSources.value.filter(s => s.strategy_id !== id);
        if (strategy.value?.id === id) {
            strategy.value = allStrategies.value[0] || null;
            if (!strategy.value) showModal.value = true;
        }
    };

    // Работа с источниками
    const openSourceMenu = async (e: MouseEvent, id: number) => {
        e.preventDefault();
        sourceIdToDelete.value = id;
        menuX.value = e.clientX;
        menuY.value = e.clientY;
        menuVisible.value = true;
        await nextTick();
        const close = () => { menuVisible.value = false; window.removeEventListener('click', close); };
        window.addEventListener('click', close);
    };

    const confirmSourceDelete = async () => {
        if (sourceIdToDelete.value === null) return;
        try {
            await invoke('delete_source', { id: sourceIdToDelete.value });
            allSources.value = allSources.value.filter(s => s.id !== sourceIdToDelete.value);
        } catch (e) {
            console.error(e);
        }
        sourceMenuVisible.value = false;
    };

    const handleDeleteSource = async () => {
        if (sourceIdToDelete.value === null) return;
        await invoke('delete_source', { id: sourceIdToDelete.value });
        allSources.value = allSources.value.filter(s => s.id !== sourceIdToDelete.value);
        menuVisible.value = false;
    };

    async function handleCreateSource(payload: { name: string }) {
        if (!strategy.value) return;

        try {
            const newSource = await invoke<Source>('create_source', {
                // Теперь это будет работать благодаря rename_all в Rust
                strategy_id: strategy.value.id,
                name: payload.name,
                icon_url: ""
            });

            if (newSource) {
                allSources.value.push(newSource);
                showSourceModal.value = false;
            }
        } catch (e) {
            console.error("Ошибка при создании источника:", e);
        }
    }

    const updateSourceBalance = (sourceId: number, amount: number) => {
        const src = allSources.value.find(s => s.id === sourceId);
        if (src) {
            src.total_balance += amount;
        }
    };

</script>

<template>
    <div class="container">
        <div v-if="isLoading" class="loader">{{ $t('common.loading') }}</div>

        <div v-else class="main-content">
            <!-- Контекстное меню источников -->
            <div v-if="menuVisible" class="context-menu" :style="{ top: menuY + 'px', left: menuX + 'px' }">
                <div class="menu-item delete" @click="handleDeleteSource">
                    <svg xmlns="http://w3.org" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2M10 11v6M14 11v6"/></svg>
                    <span>{{ $t('common.delete') }}</span>
                </div>
            </div>

            <!-- Модалки -->
            <Transition name="modal-fade">
                <div v-if="showModal" class="modal-overlay" @click.self="showModal = false">
                    <CreateStrategyModal @create="handleCreateStrategy"/>
                </div>
            </Transition>

            <Transition name="modal-fade">
                <div v-if="showSourceModal" class="modal-overlay" @click.self="showSourceModal = false">
                    <CreateSourceModal @create="handleCreateSource" @close="showSourceModal = false"/>
                </div>
            </Transition>

            <header class="header-row">
                <div class="header-left">
                    <div class="strategy-info" v-if="strategy"><h1>{{ strategy.name }}</h1></div>
                    <StrategySelector
                        :all-strategies="allStrategies"
                        :active-strategy="strategy"
                        @select="s => strategy = s"
                        @delete="handleDeleteStrategy"
                        @refresh="loadActiveStrategy"
                        @open-create="showModal = true"
                    />
                </div>
                <div class="header-right"><LanguageSwitcher /></div>
            </header>

            <main class="workspace">
                <div class="sources-section" v-if="strategy">
                    <div class="sources-frame">
                        <!-- Подпись блока -->
                        <span class="frame-label">{{ $t('strategy.sources') }}</span>

                        <div class="sources-row">
                            <template v-if="filteredSources.length > 0">
                                <SourceCard
                                    v-for="s in filteredSources"
                                    :key="s.id"
                                    :source="s"
                                    @contextmenu="openSourceMenu($event, s.id)"
                                    @deposit="openTransactionModal(s, 'deposit')"
                                    @withdraw="openTransactionModal(s, 'withdraw')"
                                />

                                <!-- Модалка транзакции -->
                                <Transition name="modal-fade">
                                    <div v-if="showTxModal" class="modal-overlay" @click.self="showTxModal = false">
                                        <TransactionModal
                                            :source-name="activeSourceForTx?.name || ''"
                                            :mode="txMode"
                                            @close="showTxModal = false"
                                            @submit="handleTransactionSubmit"
                                        />
                                    </div>
                                </Transition>

                                <Teleport to="body">
                                    <Transition name="fade-scale">
                                        <div v-if="sourceMenuVisible"
                                             class="context-menu"
                                             :style="{ top: menuY + 'px', left: menuX + 'px' }">
                                            <div class="menu-item delete" @click="confirmSourceDelete">
                                                <i class="pi pi-trash" style="font-size: 1rem"></i>
                                                <span>{{ $t('common.delete') }}</span>
                                            </div>
                                        </div>
                                    </Transition>
                                </Teleport>

                                <Teleport to="body">
                                    <div v-if="menuVisible" class="context-menu" :style="{ top: menuY + 'px', left: menuX + 'px' }">
                                        <div class="menu-item" @click="openSourceRename">
                                            <i class="pi pi-pencil"></i>
                                            <span>{{ $t('common.rename') }}</span>
                                        </div>
                                        <div class="menu-divider"></div>
                                        <div class="menu-item delete" @click="handleDeleteSource">
                                            <i class="pi pi-trash"></i>
                                            <span>{{ $t('common.delete') }}</span>
                                        </div>
                                    </div>

                                    <!-- Модалка переименования источника -->
                                    <Transition name="modal-fade">
                                        <div v-if="showSourceRenameModal" class="modal-overlay" @click.self="showSourceRenameModal = false">
                                            <div class="modal-card">
                                                <h2>{{ $t('common.rename') }}</h2>
                                                <input
                                                    v-model="newSourceName"
                                                    @keyup.enter="handleSourceRename"
                                                    autofocus
                                                    class="rename-input"
                                                    :placeholder="$t('source.placeholder')"
                                                >
                                                <div class="modal-actions">
                                                    <button @click="showSourceRenameModal = false" class="btn-cancel">{{ $t('common.cancel') }}</button>
                                                    <button @click="handleSourceRename" class="btn-submit" :disabled="!newSourceName.trim()">
                                                        {{ $t('common.save') }}
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                    </Transition>
                                </Teleport>

                                <div class="empty-source-card esc" @click="showSourceModal = true">
                                    <div class="plus-icon">+</div>
                                </div>
                            </template>
                            <div v-else class="empty-source-card" @click="showSourceModal = true">
                                <div class="plus-icon">+</div>
                                <span>{{ $t('strategy.addFirstSource') }}</span>
                            </div>
                        </div>
                    </div>
                </div>
            </main>
        </div>
    </div>
</template>

<style>
    html, body, #app { margin: 0; padding: 0; height: 100%; width: 100%; overflow: hidden; background: #000; font-family: sans-serif; }
    .container { height: 100%; width: 100%; display: flex; }
    .main-content { display: flex; flex-direction: column; height: 100%; width: 100%; background: #0f1113; }

    .header-row { display: flex; justify-content: space-between; align-items: center; height: 55px; background: #1a1d21; border-bottom: 1px solid #2d333b; }
    .header-left { display: flex; align-items: center; gap: 40px; margin-left: 25px;}
    .header-right {
        margin-right: 25px;
    }
    .strategy-info h1 { font-size: 1.1rem; color: #fff; margin: 0; min-width: 140px; }

    .workspace { flex: 1; padding: 24px; display: flex; flex-direction: column; gap: 24px; overflow-y: auto; }
    .sources-section { display: flex; flex-direction: column; gap: 12px; }
    .section-header { display: flex; align-items: center; gap: 12px; }
    .section-header h2 { font-size: 0.9rem; color: #94a3b8; text-transform: uppercase; margin: 0; }

    .sources-row {
        display: flex;
        flex-direction: row;
        gap: 12px;
        overflow-x: auto;
        align-items: center;
    }

    .empty-source-card {
        width: 7vw; height: 110px; border: 2px solid #2d333b; border-radius: 8px;
        display: flex; flex-direction: column; align-items: center; justify-content: center;
        cursor: pointer; color: #444c56; gap: 8px; transition: 0.3s; flex-shrink: 0;
    }
    .empty-source-card:hover { border-color: #00c087; color: #00c087; }
    .empty-source-card span {
        font-size: 1rem;
        text-align: center;
        padding: 0 8px;
        width: 100%;
        box-sizing: border-box;
        color: #dddddd;
    }
    .empty-source-card:hover span {
        color: #00c087;
    }
    .esc {
        width: 95px;
        height: 95px;
    }
    .sources-frame {
        position: relative;
        border: 1px solid #2d333b; /* Тонкая рамка как у карточек */
        border-radius: 12px;
        padding: 20px 15px 15px 15px;
        margin-top: 10px;
    }

    .frame-label {
        position: absolute;
        top: -10px; /* Выносим на линию рамки */
        left: 20px;
        background: #0f1113; /* Цвет фона основного окна, чтобы "перекрыть" линию */
        padding: 0 10px;
        color: #444c56;
        font-size: 0.7rem;
        font-weight: bold;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .add-source-inline {
        width: 32px;
        height: 32px;
        border-radius: 50%;
        background: transparent;
        border: 1px dashed #2d333b;
        color: #444c56;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: 0.2s;
        flex-shrink: 0;
    }

    .add-source-inline:hover {
        border-color: #00c087;
        color: #00c087;
        background: rgba(0, 192, 135, 0.05);
    }

    .add-source-btn { background: transparent; border: 1px solid #2d333b; color: #444c56; border-radius: 4px; cursor: pointer; }
    .add-source-btn:hover { border-color: #00c087; color: #00c087; }

    .context-menu { position: fixed; background: #1a1d21; border: 1px solid #444c56; border-radius: 8px; padding: 6px 0; z-index: 10000; box-shadow: 0 8px 16px rgba(0,0,0,0.5); min-width: 150px; }
    .menu-item { padding: 10px 14px; font-size: 13px; color: #94a3b8; cursor: pointer; display: flex; align-items: center; gap: 8px; }
    .menu-item:hover { background: #2d333b; color: #fff; }
    .menu-item.delete { color: #f87171; }

    .modal-overlay { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.75); display: flex; justify-content: center; align-items: center; z-index: 999; backdrop-filter: blur(8px); }
    .modal-fade-enter-active, .modal-fade-leave-active { transition: opacity 0.3s ease; }
    .modal-fade-enter-from, .modal-fade-leave-to { opacity: 0; }

    .loader { width: 100%; height: 100%; display: flex; justify-content: center; align-items: center; color: #444c56; }
    .modal-overlay {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.8); /* Темный фон */
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 99999;
        backdrop-filter: blur(8px); /* Размытие фона */
    }

    /* Сама карточка модалки */
    .modal-card {
        background: #1a1d21;
        padding: 30px;
        border-radius: 16px;
        border: 1px solid #2d333b;
        width: 100%;
        max-width: 380px;
        box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
        text-align: center;
    }

    .modal-card h2 {
        color: white;
        margin: 0 0 20px 0;
        font-size: 1.25rem;
    }

    /* Стили для инпута переименования */
    .rename-input {
        width: 100%;
        padding: 14px;
        background: #0f1113;
        border: 1px solid #2d333b;
        border-radius: 10px;
        color: white;
        margin-bottom: 25px;
        font-size: 1rem;
        outline: none;
        box-sizing: border-box;
        transition: border-color 0.2s;
    }

    .rename-input:focus {
        border-color: #00c087;
    }

    /* Кнопки в модалке */
    .modal-actions {
        display: flex;
        gap: 12px;
    }

    .btn-submit {
        flex: 1;
        background: #00c087;
        color: black;
        border: none;
        padding: 12px;
        border-radius: 10px;
        font-weight: bold;
        cursor: pointer;
        transition: 0.2s;
    }

    .btn-submit:disabled {
        opacity: 0.4;
        cursor: not-allowed;
    }

    .btn-cancel {
        flex: 1;
        background: transparent;
        color: #94a3b8;
        border: 1px solid #2d333b;
        padding: 12px;
        border-radius: 10px;
        cursor: pointer;
        transition: 0.2s;
    }

    .btn-cancel:hover {
        background: rgba(255, 255, 255, 0.05);
        color: white;
    }

    /* Анимация появления */
    .modal-fade-enter-active, .modal-fade-leave-active {
        transition: opacity 0.3s ease;
    }
    .modal-fade-enter-from, .modal-fade-leave-to {
        opacity: 0;
    }
</style>
