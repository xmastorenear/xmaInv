<script setup lang="ts">
    import AssetTypeCard from "./AssetTypeCard.vue";
    import type { AssetGroup, Asset } from "../../types";

    defineProps<{
        assetGroups: AssetGroup[],
        allAssets: Asset[],
        expandedId: number | null,
        marketPrices?: Record<number, { price: number; stale: boolean }>,
        pricesLoading?: boolean
    }>();

    const emit = defineEmits<{
        (e: 'add-group'): void;
        (e: 'expand', id: number): void;
        (e: 'delete', id: number): void;
        (e: 'add-asset', id: number): void;
        (e: 'delete-asset', assetId: number): void;
        (e: 'open-menu', event: MouseEvent, id: number): void;
        (e: 'buy-more', payload: any): void;
        (e: 'sell-asset', payload: any): void;
        (e: 'add-new-asset', payload: any): void;
        (e: 'refresh-prices'): void;
    }>();
</script>

<template>
    <div class="assets-section">
        <Transition name="fade">
            <div v-if="expandedId" class="expand-overlay" @click="emit('expand', expandedId)"></div>
        </Transition>

        <div class="assets-frame" :class="{ 'has-expanded': expandedId }">
            <span class="frame-label">
                {{ $t('strategy.assetGroups') }}
                <button v-if="!expandedId" class="add-asset-btn" @click="emit('add-group')">
                    <i class="pi pi-plus"></i>
                </button>
            </span>

            <div class="assets-grid">
                <template v-if="assetGroups.length > 0">
                    <div
                        v-for="group in assetGroups"
                        :key="group.id"
                        @contextmenu.prevent="e => emit('open-menu', e, group.id)"
                        class="asset-wrapper"
                    >
                        <Teleport to="body" :disabled="expandedId !== group.id">
                            <AssetTypeCard
                                :asset="group"
                                :is-expanded="expandedId === group.id"
                                :all-assets="allAssets"
                                :market-prices="marketPrices"
                                :prices-loading="pricesLoading"
                                @expand="(id) => emit('expand', id)"
                                @delete="(id) => emit('delete', id)"
                                @add-asset="(id) => emit('add-asset', id)"
                                @delete-asset="(assetId) => emit('delete-asset', assetId)"
                                @buy-more="(payload) => emit('buy-more', payload)"
                                @sell-asset="(payload) => emit('sell-asset', payload)"
                                @add-new-asset="(payload) => emit('add-new-asset', payload)"
                                @refresh-prices="emit('refresh-prices')"
                            />
                        </Teleport>
                    </div>
                </template>
            </div>
        </div>
    </div>
</template>

<style scoped>
    .assets-frame {
        position: relative;
        border: 1px solid #2d333b;
        border-radius: 12px;
        padding: 24px 16px 16px 16px;
        margin-top: 20px;
    }

    .assets-frame.has-expanded {
        border-color: transparent;
    }

    .frame-label {
        position: absolute;
        top: -12px;
        left: 20px;
        background: #0f1113;
        padding: 0px 10px;
        color: #444c56;
        font-size: 0.7rem;
        font-weight: bold;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        display: flex;
        gap: 15px;
        align-items: center;
        flex-direction: row;
        flex-wrap: nowrap;
        align-content: center;
        justify-content: center;
    }

    .assets-grid {
        display: flex;
        flex-wrap: wrap;
        gap: 12px;
        align-items: stretch;
        width: 100%;
    }

    .asset-wrapper {
        flex: 1;
        min-width: 200px;
        display: flex;
    }

    :deep(.asset-type-card:not(.expanded)) {
        width: 100% !important;
        min-width: 0;
    }

    .expand-overlay {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.8);
        backdrop-filter: blur(4px);
        z-index: 1000;
        pointer-events: auto;
    }

    .add-asset-btn {
        width: 25px;
        height: 25px;
        background: transparent;
        border: 1px dashed #2d333b;
        color: #444c56;
        border-radius: 5px;
        cursor: pointer;
        transition: 0.2s;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .add-asset-btn:hover {
        border-color: #00c087;
        color: #00c087;
        background: rgba(0, 192, 135, 0.05);
    }

    .fade-enter-active, .fade-leave-active { transition: opacity 0.3s; }

    .fade-enter-from, .fade-leave-to { opacity: 0; }

    .assets-frame.has-expanded,
    .assets-frame.has-expanded .frame-label {
        opacity: 0;
        pointer-events: none;
        transition: opacity 0.2s ease;
    }
</style>
