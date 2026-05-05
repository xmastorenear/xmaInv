<script setup lang="ts">
    import SourceCard from "../SourceCard.vue";
    import type { Source } from "../../types";

    defineProps<{
        sources: Source[];
    }>();

    const emit = defineEmits<{
        (e: 'add-source'): void;
        (e: 'open-menu', event: MouseEvent, id: number): void;
        (e: 'deposit', s: Source): void;
        (e: 'withdraw', s: Source): void;
    }>();
</script>

<template>
    <div class="sources-section">
        <div class="sources-frame">
            <span class="frame-label">{{ $t('strategy.sources') }}</span>

            <div class="sources-row">
                <template v-if="sources.length > 0">
                    <SourceCard
                        v-for="s in sources"
                        :key="s.id"
                        :source="s"
                        @contextmenu="e => emit('open-menu', e, s.id)"
                        @deposit="emit('deposit', s)"
                        @withdraw="emit('withdraw', s)"
                    />

                    <button class="add-source-card inline-variant" @click="emit('add-source')">
                        <i class="pi pi-plus"></i>
                    </button>
                </template>

                <div v-else class="add-source-card empty-state-variant" @click="emit('add-source')">
                    <span>{{ $t('strategy.addFirstSource') }}</span>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
    .sources-frame {
        position: relative;
        border: 1px solid #2d333b;
        border-radius: 12px;
        padding: 20px 15px 15px 15px;
        margin-top: 10px;
    }

    .frame-label {
        position: absolute;
        top: -10px; left: 20px;
        background: #0f1113;
        padding: 0 10px;
        color: #444c56;
        font-size: 0.7rem;
        font-weight: bold;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .sources-row {
        display: flex;
        flex-direction: row;
        gap: 12px;
        overflow-x: auto;
        align-items: stretch;
    }

    .add-source-card {
        background: transparent;
        border: 2px dashed #2d333b;
        border-radius: 8px;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        color: #444c56;
        transition: 0.3s;
        flex-shrink: 0;
    }

    .inline-variant {
        width: 97px;
        height: 97px;
    }

    .empty-state-variant {
        width: 97px; height: 97px; border: 2px solid #2d333b; border-radius: 8px;
        display: flex; flex-direction: column; align-items: center; justify-content: center;
        cursor: pointer; color: #444c56; gap: 8px; transition: 0.3s; flex-shrink: 0;
    }
    .empty-state-variant span {
        font-size: 1rem;
        text-align: center;
        padding: 0 8px;
        width: 100%;
        box-sizing: border-box;
        color: #dddddd;
    }
    .empty-state-variant:hover span {
        color: #00c087;
    }

    .add-source-card:hover {
        border-color: #00c087;
        color: #00c087;
        background: rgba(0, 192, 135, 0.05);
    }

    .add-source-card i {
        font-size: 1.2rem;
    }

    .add-source-card span {
        font-size: 1rem;
        font-weight: 500;
    }
</style>
