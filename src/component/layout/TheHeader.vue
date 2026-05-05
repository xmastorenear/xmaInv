<script setup lang="ts">
import StrategySelector from "../StrategySelector.vue";
import LanguageSwitcher from "../LanguageSwitcher.vue";
import type { Strategy } from "../../types";

defineProps<{
    activeStrategy: Strategy | null;
    allStrategies: Strategy[];
}>();

const emit = defineEmits<{
    (e: 'select-strategy', s: Strategy): void;
    (e: 'delete-strategy', id: number): void;
    (e: 'open-create-strategy'): void;
    (e: 'refresh'): void;
}>();
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
            <LanguageSwitcher />
        </div>
    </header>
</template>

<style scoped>
    .header-row { display: flex; justify-content: space-between; align-items: center; height: 55px; background: #1a1d21; border-bottom: 1px solid #2d333b; }
    .header-left { display: flex; align-items: center; gap: 40px; margin-left: 25px;}
    .header-right { margin-right: 25px; }
    .strategy-info h1 { font-size: 1.1rem; color: #fff; margin: 0; min-width: 140px; }
</style>
