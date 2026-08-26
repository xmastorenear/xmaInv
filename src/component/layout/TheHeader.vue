<script setup lang="ts">
    import { ref, watch } from 'vue';
    import StrategySelector from "../StrategySelector.vue";
    import LanguageSwitcher from "../LanguageSwitcher.vue";
    import type { Strategy } from "../../types";

    const props = defineProps<{
        activeStrategy: Strategy | null;
        allStrategies: Strategy[];
        tbankToken?: string | null;
    }>();

    const emit = defineEmits<{
        (e: 'select-strategy', s: Strategy): void;
        (e: 'delete-strategy', id: number): void;
        (e: 'open-create-strategy'): void;
        (e: 'refresh'): void;
        (e: 'save-tbank-token', token: string): void;
    }>();

    const tokenInput = ref(props.tbankToken || '');
    const tokenSaved = ref(false);

    // Sync the field when data is loaded (e.g. after a refresh)
    watch(() => props.tbankToken, (val) => {
        if (val !== tokenInput.value) {
            tokenInput.value = val || '';
        }
    });

    const saveToken = () => {
        emit('save-tbank-token', tokenInput.value.trim());
        tokenSaved.value = true;
        setTimeout(() => { tokenSaved.value = false; }, 2000);
    };
</script>

<template>
    <header class="header-row">
        <div class="header-left">
            <div class="strategy-info" v-if="activeStrategy">
                <h1>{{ activeStrategy.name }}</h1>
            </div>

            <StrategySelector
                :all-strategies="allStrategies"
                :active-strategy="activeStrategy"
                @select="s => emit('select-strategy', s)"
                @delete="id => emit('delete-strategy', id)"
                @open-create="emit('open-create-strategy')"
                @refresh="emit('refresh')"
            />
        </div>

        <div class="header-right">
            <div class="token-box">
                <input
                    v-model="tokenInput"
                    type="password"
                    class="token-input"
                    :placeholder="$t('tbank.tokenPlaceholder')"
                    :title="$t('tbank.tokenTitle')"
                    @keyup.enter="saveToken"
                />
                <button
                    class="token-btn"
                    :class="{ 'token-btn-saved': tokenSaved }"
                    :disabled="!tokenInput.trim()"
                    @click="saveToken"
                >
                    {{ tokenSaved ? $t('tbank.tokenSaved') : $t('tbank.tokenSave') }}
                </button>
            </div>
            <LanguageSwitcher />
        </div>
    </header>
</template>

<style scoped>
    .header-row { display: flex; justify-content: space-between; align-items: center; height: 55px; background: #1a1d21; border-bottom: 1px solid #2d333b; }

    .header-left { display: flex; align-items: center; gap: 40px; margin-left: 25px;}

    .header-right { margin-right: 25px; display: flex; align-items: center; gap: 12px; }

    .strategy-info h1 { font-size: 1.1rem; color: #fff; margin: 0; min-width: 140px; }

    .token-box { display: flex; align-items: center; gap: 6px; }
    .token-input {
        width: 210px;
        padding: 7px 12px;
        background: #0f1113;
        border: 1px solid #2d333b;
        border-radius: 8px;
        color: #94a3b8;
        font-size: 0.8rem;
        outline: none;
        transition: border-color 0.2s;
    }
    .token-input:focus { border-color: #8b5cf6; }
    .token-btn {
        padding: 7px 12px;
        background: #8b5cf6;
        color: #fff;
        border: none;
        border-radius: 8px;
        font-size: 0.8rem;
        font-weight: bold;
        cursor: pointer;
        transition: background 0.2s;
    }
    .token-btn:disabled { opacity: 0.5; cursor: not-allowed; }
    .token-btn-saved { background: #22c55e; }

</style>
