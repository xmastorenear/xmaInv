import {ref, computed, type Ref} from 'vue';
import { invoke } from "@tauri-apps/api/core";
import type { AssetGroup, Strategy, Asset } from "../../types";

export function useAssetGroups(activeStrategy: Ref<Strategy | null>, onAssetsChanged?: () => void) {
    const allAssetGroups = ref<AssetGroup[]>([]);
    const allAssets = ref<Asset[]>([]);

    const showAddAssetModal = ref(false);
    const selectedGroupIdForNewAsset = ref<number | null>(null);
    const showAssetGroupModal = ref(false);


    const filteredAssetGroups = computed(() => {
        if (!activeStrategy.value) return [];
        return allAssetGroups.value.filter(g => g.strategy_id === activeStrategy.value!.id);
    });

    const handleCreateAssetGroup = async (payload: { name: string, distribution: Record<number, number>, new_group_percent: number }) => {
        if (!activeStrategy.value || !payload.name.trim()) return;

        try {
            // TYPING MANEUVER: Convert JS object string keys into numeric types for Rust HashMap<u32, f64>
            const cleanDistribution: Record<number, number> = {};
            Object.entries(payload.distribution).forEach(([key, val]) => {
                cleanDistribution[Number(key)] = Number(val);
            });

            console.log("[DEBUG] Sending group creation data to Rust:", {
                strategy_id: Number(activeStrategy.value.id),
                name: payload.name.trim(),
                distribution: cleanDistribution,
                new_group_percent: Number(payload.new_group_percent)
            });

            // Invoke the Tauri command
            const res = await invoke<AssetGroup>('create_asset_group', {
                strategy_id: Number(activeStrategy.value.id),
                name: payload.name.trim(),
                distribution: cleanDistribution, // Clean map with numeric keys
                new_group_percent: Number(payload.new_group_percent)
            });

            allAssetGroups.value.push(res);
            console.log("[DEBUG] Group created successfully on the backend:", res);

        } catch (e) {
            console.error("Error sending group to Rust:", e);
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
    const handleCreateAsset = async (payload: { group_id: number; ticker: string; amount: number; price: number; uid?: string | null }) => {
        try {
            console.log("[DEBUG] Sending new asset via create_asset to Rust:", payload);

            // Use the correct create_asset command to create a new asset
            const newAsset = await invoke<Asset>('create_asset', {
                group_id: Number(payload.group_id),
                ticker: payload.ticker.trim().toUpperCase(),
                amount: Number(payload.amount),
                buy_price: Number(payload.price),
                uid: payload.uid ?? null
            });

            console.log("[DEBUG] Backend created the asset:", newAsset);

            const index = allAssets.value.findIndex(a => Number(a.id) === Number(newAsset.id));
            if (index !== -1) {
                allAssets.value[index] = newAsset;
            } else {
                allAssets.value.push(newAsset);
            }
            allAssets.value = [...allAssets.value];

            onAssetsChanged?.();

        } catch (e) {
            console.error("Error adding asset via create_asset:", e);
        }
    };
    const handleDeleteAsset = async (assetId: number) => {
        try {
            await invoke('delete_asset', { id: assetId });

            allAssets.value = allAssets.value.filter(a => a.id !== assetId);

            onAssetsChanged?.();

            console.log(`[DEBUG] Asset ${assetId} deleted successfully`);
        } catch (e) {
            console.error("Error deleting asset:", e);
        }
    };

    const handleBuyMoreAsset = async (payload: { asset_id: number; amount: number; price: number }) => {
        try {
            console.log("[DEBUG] Sending buy-more data to Rust:", payload);

            const updatedAsset = await invoke<Asset>('buy_more_asset', {
                id: Number(payload.asset_id),
                added_amount: Number(payload.amount),
                execution_price: Number(payload.price)
            });

            const index = allAssets.value.findIndex(a => Number(a.id) === Number(updatedAsset.id));
            if (index !== -1) {
                allAssets.value[index] = updatedAsset;
            } else {
                allAssets.value.push(updatedAsset);
            }
            allAssets.value = [...allAssets.value];
            console.log("[DEBUG] Asset updated with the server value:", updatedAsset);

            onAssetsChanged?.();
        } catch (e) {
            console.error("Error in handleBuyMoreAsset:", e);
        }
    };
    const handleSellAsset = async (payload: { asset_id: number; amount: number; price: number }) => {
        try {
            const result = await invoke<Asset | null>('sell_asset', {
                id: Number(payload.asset_id),
                sell_amount: Number(payload.amount),
                execution_price: Number(payload.price)
            });

            if (result === null) {
                allAssets.value = allAssets.value.filter(a => Number(a.id) !== Number(payload.asset_id));
                console.log(`[DEBUG] Asset ${payload.asset_id} fully sold and removed.`);
            } else {
                const index = allAssets.value.findIndex(a => Number(a.id) === Number(result.id));
                if (index !== -1) {
                    allAssets.value[index] = result;
                    allAssets.value = [...allAssets.value];
                }
                console.log(`[DEBUG] Partial sell. Remaining quantity: ${result.amount}`);
            }
            onAssetsChanged?.();
        } catch (e) {
            console.error("Error selling asset:", e);
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