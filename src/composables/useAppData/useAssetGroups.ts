import {ref, computed, type Ref} from 'vue';
import { invoke } from "@tauri-apps/api/core";
import type { AssetGroup, Strategy, Asset } from "../../types";

export function useAssetGroups(activeStrategy: Ref<Strategy | null>) {
    const allAssetGroups = ref<AssetGroup[]>([]);
    const allAssets = ref<Asset[]>([]);

    const showAddAssetModal = ref(false);
    const selectedGroupIdForNewAsset = ref<number | null>(null);
    const showAssetGroupModal = ref(false);


    const filteredAssetGroups = computed(() => {
        if (!activeStrategy.value) return [];
        return allAssetGroups.value.filter(g => g.strategy_id === activeStrategy.value!.id);
    });

    const handleCreateAssetGroup = async (payload: { name: string }) => {
        const name = payload.name;
        if (!activeStrategy.value || !name.trim()) return;

        try {
            const res = await invoke<AssetGroup>('create_asset_group', {
                strategy_id: Number(activeStrategy.value.id),
                name: name.trim()
            });
            allAssetGroups.value.push(res);
        } catch (e) {
            console.error("Ошибка при отправке группы в Rust:", e);
        } finally {
            showAssetGroupModal.value = false;
        }
    };
    const handleDeleteAssetGroup = async (id: number) => {
        try {
            await invoke('delete_asset_group', { id });
            allAssetGroups.value = allAssetGroups.value.filter(g => g.id !== id);
        } catch (e) {
            console.error("Delete group error:", e);
        }
    };
    const handleCreateAsset = async (payload: { ticker: string; amount: number; buy_price: number }) => {
        if (selectedGroupIdForNewAsset.value === null) return;
        try {
            const res = await invoke<Asset>('create_asset', {
                group_id: Number(selectedGroupIdForNewAsset.value),
                ticker: payload.ticker,
                amount: Number(payload.amount),
                buy_price: Number(payload.buy_price)
            });
            allAssets.value.push(res);

            const targetGroup = allAssetGroups.value.find(g => g.id === selectedGroupIdForNewAsset.value);
            if (targetGroup) {
                targetGroup.total_value += payload.amount * payload.buy_price;
            }
        } catch (e) {
            console.error("Ошибка при отправке актива в Rust:", e);
        } finally {
            showAddAssetModal.value = false;
            selectedGroupIdForNewAsset.value = null;
        }
    };
    const handleDeleteAsset = async (assetId: number) => {
        try {
            await invoke('delete_asset', { id: assetId });

            allAssets.value = allAssets.value.filter(a => a.id !== assetId);

            console.log(`[DEBUG] Актив ${assetId} успешно удален`);
        } catch (e) {
            console.error("Ошибка удаления актива:", e);
        }
    };

    const handleBuyMoreAsset = async (payload: { asset_id: number; amount: number; price: number }) => {
        try {
            console.log("[DEBUG] Отправка данных докупки в Rust:", payload);

            await invoke('buy_more_asset', {
                id: Number(payload.asset_id),
                added_amount: Number(payload.amount),
                execution_price: Number(payload.price)
            });

            const target = allAssets.value.find(a => Number(a.id) === Number(payload.asset_id));

            if (target) {
                const currentTotalCost = Number(target.amount) * Number(target.buy_price);
                const newTotalCost = Number(payload.amount) * Number(payload.price);

                target.amount = Number(target.amount) + Number(payload.amount);
                target.buy_price = (currentTotalCost + newTotalCost) / target.amount;

                allAssets.value = [...allAssets.value];
                console.log("[DEBUG] Массив allAssets успешно обновлен в памяти:", target);
            } else {
                console.warn(`[DEBUG] Актив с ID ${payload.asset_id} не найден в локальном массиве allAssets!`);
            }
        } catch (e) {
            console.error("Ошибка при выполнении handleBuyMoreAsset:", e);
        }
    };
    const handleSellAsset = async (payload: { asset_id: number; amount: number; price: number }) => {
        try {
            await invoke('sell_asset', {
                id: Number(payload.asset_id),
                sell_amount: Number(payload.amount),
                execution_price: Number(payload.price)
            });

            const targetIndex = allAssets.value.findIndex(a => Number(a.id) === Number(payload.asset_id));

            if (targetIndex !== -1) {
                const target = allAssets.value[targetIndex];

                if (target.amount <= payload.amount) {
                    allAssets.value.splice(targetIndex, 1);
                    console.log(`[DEBUG] Актив ${payload.asset_id} полностью продан и удален.`);
                } else {
                    target.amount -= payload.amount;
                    console.log(`[DEBUG] Частичная продажа. Остаток количества: ${target.amount}`);
                }

                allAssets.value = [...allAssets.value];
            }
        } catch (e) {
            console.error("Ошибка при продаже актива:", e);
        }
    };

    return {
        allAssetGroups,
        allAssets,
        filteredAssetGroups,
        showAssetGroupModal,
        showAddAssetModal,
        selectedGroupIdForNewAsset,
        handleCreateAssetGroup,
        handleDeleteAssetGroup,
        handleCreateAsset,
        handleDeleteAsset,
        handleBuyMoreAsset,
        handleSellAsset
    };
}