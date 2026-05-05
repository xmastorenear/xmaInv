<script setup lang="ts">
import { onMounted } from "vue";
import { useAppData } from "./composables/useAppData/index.ts";

import TheHeader from "./component/layout/TheHeader.vue";
import SourcesBlock from "./component/sources/SourcesBlock.vue";

import CreateStrategyModal from "./component/CreateStrategyModal.vue";
import CreateSourceModal from "./component/CreateSourceModal.vue";
import TransactionModal from "./component/TransactionModal.vue";

const {
    strategy,
    allStrategies,
    filteredSources,
    isLoading,
    loadData,
    showModal,
    showSourceModal,
    showSourceRenameModal,
    showTxModal,
    menuVisible, menuX, menuY,
    openSourceMenu,
    handleDeleteSource,
    openSourceRename,
    newSourceName,
    handleSourceRename,
    txMode,
    activeSourceForTx,
    openTransactionModal,
    handleTransactionSubmit,
    handleCreateStrategy,
    handleDeleteStrategy,
    handleCreateSource,
    handleRenameStrategy
} = useAppData();

onMounted(loadData);
</script>

<template>
    <div class="container">
        <div v-if="isLoading" class="loader">{{ $t('common.loading') }}</div>

        <div v-else class="main-content">
            <Teleport to="body">
                <Transition name="fade-scale">
                    <div v-if="menuVisible"
                         class="context-menu"
                         :style="{ top: menuY + 'px', left: menuX + 'px' }">
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
                </Transition>

                <Transition name="modal-fade">
                    <div v-if="showModal" class="modal-overlay" @click.self="showModal = false">
                        <CreateStrategyModal @create="handleCreateStrategy"/>
                    </div>
                </Transition>

                <Transition name="modal-fade">
                    <div v-if="showSourceModal" class="modal-overlay" @click.self="showSourceModal = false">
                        <CreateSourceModal
                            @create="handleCreateSource"
                            @close="showSourceModal = false"
                        />
                    </div>
                </Transition>

                <Transition name="modal-fade">
                    <div v-if="showSourceRenameModal" class="modal-overlay" @click.self="showSourceRenameModal = false">
                        <div class="modal-card">
                            <h2>{{ $t('common.rename') }}</h2>
                            <input v-model="newSourceName" @keyup.enter="handleSourceRename" autofocus class="rename-input">
                            <div class="modal-actions">
                                <button @click="showSourceRenameModal = false" class="btn-cancel">{{ $t('common.cancel') }}</button>
                                <button @click="handleSourceRename" class="btn-submit" :disabled="!newSourceName.trim()">
                                    {{ $t('common.save') }}
                                </button>
                            </div>
                        </div>
                    </div>
                </Transition>

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
            </Teleport>

            <TheHeader
                :active-strategy="strategy"
                :all-strategies="allStrategies"
                @select-strategy="s => strategy = s"
                @delete-strategy="handleDeleteStrategy"
                @rename-strategy="handleRenameStrategy"
                @open-create-strategy="showModal = true"
                @refresh="loadData"
            />

            <main class="workspace">
                <SourcesBlock
                    v-if="strategy"
                    :sources="filteredSources"
                    @open-menu="openSourceMenu"
                    @add-source="showSourceModal = true"
                    @deposit="s => openTransactionModal(s, 'deposit')"
                    @withdraw="s => openTransactionModal(s, 'withdraw')"
                />

                <div v-else class="empty-state">
                    {{ $t('strategy.selectToStart') }}
                </div>
            </main>
        </div>
    </div>
</template>

<style>

    html, body, #app { margin: 0; padding: 0; height: 100%; width: 100%; overflow: hidden; background: #000; font-family: sans-serif; }
    .container { height: 100%; width: 100%; display: flex; }
    .main-content { display: flex; flex-direction: column; height: 100%; width: 100%; background: #0f1113; }
    .workspace { flex: 1; padding: 24px; display: flex; flex-direction: column; gap: 24px; overflow-y: auto; }


    .context-menu {
        position: fixed; background: #1a1d21; border: 1px solid #444c56;
        border-radius: 8px; padding: 6px 0; z-index: 100000;
        box-shadow: 0 8px 16px rgba(0,0,0,0.5); min-width: 150px;
    }
    .menu-item { padding: 10px 14px; font-size: 13px; color: #94a3b8; cursor: pointer; display: flex; align-items: center; gap: 8px; }
    .menu-item:hover { background: #2d333b; color: #fff; }
    .menu-item.delete { color: #f87171; }
    .menu-divider { height: 1px; background: #2d333b; margin: 4px 8px; }


    .modal-overlay { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.8); display: flex; justify-content: center; align-items: center; z-index: 99999; backdrop-filter: blur(8px); }
    .modal-card { background: #1a1d21; padding: 30px; border-radius: 16px; border: 1px solid #2d333b; width: 100%; max-width: 380px; box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5); text-align: center; }
    .modal-card h2 { color: white; margin: 0 0 20px 0; font-size: 1.25rem; }

    .rename-input { width: 100%; padding: 14px; background: #0f1113; border: 1px solid #2d333b; border-radius: 10px; color: white; margin-bottom: 25px; font-size: 1rem; outline: none; box-sizing: border-box; }
    .rename-input:focus { border-color: #00c087; }

    .modal-actions { display: flex; gap: 12px; }
    .btn-submit { flex: 1; background: #00c087; color: black; border: none; padding: 12px; border-radius: 10px; font-weight: bold; cursor: pointer; }
    .btn-cancel { flex: 1; background: transparent; color: #94a3b8; border: 1px solid #2d333b; padding: 12px; border-radius: 10px; cursor: pointer; }


    .modal-fade-enter-active, .modal-fade-leave-active { transition: opacity 0.3s ease; }
    .modal-fade-enter-from, .modal-fade-leave-to { opacity: 0; }
    .fade-scale-enter-active, .fade-scale-leave-active { transition: all 0.15s ease; }
    .fade-scale-enter-from, .fade-scale-leave-to { opacity: 0; transform: scale(0.95); }

    .loader { width: 100%; height: 100%; display: flex; justify-content: center; align-items: center; color: #444c56; }
</style>
