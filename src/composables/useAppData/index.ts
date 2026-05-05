import { ref } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import type { AppDataResponse } from "../../types";

import { useStrategies } from "./useStrategies";
import { useSources } from "./useSources";
import { useAssetGroups } from "./useAssetGroups";
import { useTransactions } from "./useTransactions";

export function useAppData() {
    const isLoading = ref(true);

    const strategies = useStrategies();
    // Передаем активную стратегию в модули, которые от нее зависят
    const sources = useSources(strategies.strategy);
    const assetGroups = useAssetGroups(strategies.strategy);
    const transactions = useTransactions(sources.allSources);

    const loadData = async () => {
        try {
            const data = await invoke<AppDataResponse>('get_data');

            strategies.allStrategies.value = data.all_strategies || [];
            sources.allSources.value = data.sources || [];
            assetGroups.allAssetGroups.value = data.asset_groups || [];
            assetGroups.allAssets.value = data.assets || [];

            if (data.strategy) {
                strategies.strategy.value = data.strategy;
            } else if (strategies.allStrategies.value.length > 0) {
                strategies.strategy.value = strategies.allStrategies.value[0];
            } else {
                strategies.strategy.value = null;
                strategies.showModal.value = true;
            }
        } catch (e) {
            console.error("Ошибка загрузки данных:", e);
        } finally {
            isLoading.value = false;
        }
    };

    return {
        isLoading,
        loadData,

        strategy: strategies.strategy,
        allStrategies: strategies.allStrategies,
        showModal: strategies.showModal,
        handleCreateStrategy: strategies.handleCreateStrategy,
        handleDeleteStrategy: strategies.handleDeleteStrategy,
        handleRenameStrategy: strategies.handleRenameStrategy,

        allSources: sources.allSources,
        filteredSources: sources.filteredSources,
        showSourceModal: sources.showSourceModal,
        showSourceRenameModal: sources.showSourceRenameModal,
        newSourceName: sources.newSourceName,
        sourceMenuVisible: sources.sourceMenuVisible, // Переименовано для ясности
        sourceMenuX: sources.sourceMenuX,
        sourceMenuY: sources.sourceMenuY,
        openSourceMenu: sources.openSourceMenu,
        handleCreateSource: sources.handleCreateSource,
        handleDeleteSource: sources.handleDeleteSource,
        openSourceRename: sources.openSourceRename,
        handleSourceRename: sources.handleSourceRename,

        allAssetGroups: assetGroups.allAssetGroups,
        allAssets: assetGroups.allAssets,
        filteredAssetGroups: assetGroups.filteredAssetGroups,
        showAssetGroupModal: assetGroups.showAssetGroupModal,
        handleCreateAssetGroup: assetGroups.handleCreateAssetGroup,
        handleDeleteAssetGroup: assetGroups.handleDeleteAssetGroup,
        handleDeleteAsset: assetGroups.handleDeleteAsset,

        showAddAssetModal: assetGroups.showAddAssetModal,
        selectedGroupIdForNewAsset: assetGroups.selectedGroupIdForNewAsset,
        handleCreateAsset: assetGroups.handleCreateAsset,

        showTxModal: transactions.showTxModal,
        txMode: transactions.txMode,
        activeSourceForTx: transactions.activeSourceForTx,
        openTransactionModal: transactions.openTransactionModal,
        handleTransactionSubmit: transactions.handleTransactionSubmit,
        handleBuyMoreAsset: assetGroups.handleBuyMoreAsset,
        handleSellAsset: assetGroups.handleSellAsset
    };
}
