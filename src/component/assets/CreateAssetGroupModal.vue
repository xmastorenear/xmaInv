<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import type { AssetGroup, Strategy } from "../../types";

const props = defineProps<{
    activeStrategy: Strategy | null;
    existingGroups: AssetGroup[];
}>();

const emit = defineEmits<{
    (e: 'close'): void;
    (e: 'create', payload: { name: string, distribution: Record<number, number>, new_group_percent: number }): void;
}>();

const groupName = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

// Percentage distribution map for existing portfolio groups
const distribution = ref<Record<number, number>>({});
// Percentage for the newly created group
const newGroupPercent = ref(0);

onMounted(() => {
    inputRef.value?.focus();

    // Calculate even shares at startup (sum is exactly 100%)
    const totalItemsCount = props.existingGroups.length + 1;
    const baseShare = Math.floor(100 / totalItemsCount);
    let remainder = 100 - (baseShare * totalItemsCount);

    newGroupPercent.value = baseShare + (remainder > 0 ? 1 : 0);
    if (remainder > 0) remainder--;

    props.existingGroups.forEach(g => {
        const bonus = remainder > 0 ? 1 : 0;
        distribution.value[g.id] = baseShare + bonus;
        if (remainder > 0) remainder--;
    });
});

// CASCADE RECALC: When the TOPMOST slider changes (New group)
const handleNewGroupChange = (value: number) => {
    // Clamp the input
    let targetVal = Math.max(0, Math.min(100, value));
    newGroupPercent.value = targetVal;

    // Count how much remains to distribute among the lower items
    let remainingAmount = 100 - targetVal;
    const lowerGroups = props.existingGroups;

    if (lowerGroups.length === 0) return;

    // Distribute the remainder evenly among all lower items
    const baseShare = Math.floor(remainingAmount / lowerGroups.length);
    let remainder = remainingAmount - (baseShare * lowerGroups.length);

    lowerGroups.forEach(g => {
        const bonus = remainder > 0 ? 1 : 0;
        distribution.value[g.id] = baseShare + bonus;
        if (remainder > 0) remainder--;
    });
};

// CASCADE RECALC: When one of the EXISTING sliders changes
const handleExistingGroupChange = (index: number, groupId: number, value: number) => {
    let targetVal = Math.max(0, Math.min(100, value));
    distribution.value[groupId] = targetVal;

    // Sum of all upper ("frozen") portfolio items
    const upperSum = newGroupPercent.value + props.existingGroups
        .slice(0, index)
        .reduce((sum, g) => sum + (distribution.value[g.id] || 0), 0);

    // How many percent are available to distribute among the lower items
    let remainingAmount = 100 - upperSum - targetVal;

    // Cut only the groups that are strictly below the current one
    const lowerGroups = props.existingGroups.slice(index + 1);

    if (lowerGroups.length > 0) {
        // Protection: if the remainder is negative, shrink the lower items to zero
        if (remainingAmount < 0) {
            lowerGroups.forEach(g => distribution.value[g.id] = 0);
            return;
        }

        const baseShare = Math.floor(remainingAmount / lowerGroups.length);
        let remainder = remainingAmount - (baseShare * lowerGroups.length);

        lowerGroups.forEach(g => {
            const bonus = remainder > 0 ? 1 : 0;
            distribution.value[g.id] = baseShare + bonus;
            if (remainder > 0) remainder--;
        });
    }
};

// Sum of all sliders on the form for indicator validation
const totalPercent = computed(() => {
    const oldSum = Object.values(distribution.value).reduce((sum, val) => sum + (val || 0), 0);
    return oldSum + (newGroupPercent.value || 0);
});

const isFormValid = computed(() => {
    return groupName.value.trim() && totalPercent.value === 100;
});

const handleSubmit = () => {
    if (!isFormValid.value) return;
    emit('create', {
        name: groupName.value.trim(),
        distribution: distribution.value,
        new_group_percent: newGroupPercent.value
    });
};
</script>



<template>
    <div class="modal-card">
        <header class="modal-header-box">
            <div class="modal-icon"><i class="pi pi-box"></i></div>
            <h2>Новая группа активов</h2>
        </header>

        <div class="form-group">
            <label class="input-title">Название новой группы</label>
            <input
                ref="inputRef"
                v-model="groupName"
                type="text"
                placeholder="Например: Акции РФ, Крипта..."
                class="modal-input"
            >
        </div>

        <div class="investment-plan-box">
            <div class="plan-header">
                <h3>Доли распределения капитала</h3>
                <span class="percent-badge" :class="{ 'valid': totalPercent === 100 }">
                    {{ totalPercent }}% / 100%
                </span>
            </div>

            <div class="sliders-list">
                <!-- Topmost slider: New group (index is conditionally -1, recalculates all the lower ones) -->
                <div class="slider-row new-group-row">
                    <span class="g-name-label">
                        {{ groupName.trim() ? groupName.trim() : 'Новая группа' }}
                    </span>
                    <div class="slider-controls">
                        <input
                            :value="newGroupPercent"
                            type="range"
                            min="0"
                            max="100"
                            class="neon-slider"
                            @input="e => handleNewGroupChange(Number((e.target as HTMLInputElement).value))"
                        >
                        <input
                            :value="newGroupPercent"
                            type="number"
                            min="0"
                            max="100"
                            class="percent-val-input"
                            @change="e => handleNewGroupChange(Number((e.target as HTMLInputElement).value))"
                        >
                        <span class="unit-text">%</span>
                    </div>
                </div>

                <!-- Sliders for existing groups (pass the row index so the upper items are left untouched) -->
                <div v-for="(g, index) in existingGroups" :key="g.id" class="slider-row">
                    <span class="g-name-label">{{ g.name }}</span>
                    <div class="slider-controls">
                        <input
                            :value="distribution[g.id] || 0"
                            type="range"
                            min="0"
                            max="100"
                            class="neon-slider"
                            @input="e => handleExistingGroupChange(index, g.id, Number((e.target as HTMLInputElement).value))"
                        >
                        <input
                            :value="distribution[g.id] || 0"
                            type="number"
                            min="0"
                            max="100"
                            class="percent-val-input"
                            @change="e => handleExistingGroupChange(index, g.id, Number((e.target as HTMLInputElement).value))"
                        >
                        <span class="unit-text">%</span>
                    </div>
                </div>
            </div>
        </div>

        <div class="modal-actions">
            <button class="btn-cancel" @click="emit('close')">Отмена</button>
            <button class="btn-submit" :disabled="!isFormValid" @click="handleSubmit">Создать</button>
        </div>
    </div>
</template>



<style scoped>
    .modal-card {
        background: #241f33;
        border: 1px solid #6d5bd0;
        border-radius: 12px;
        padding: 24px;
        width: 100%;
        max-width: 450px;
        box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
    }
    .modal-header-box {
        display: flex;
        align-items: center;
        gap: 12px;
        margin-bottom: 24px;
        border-bottom: 1px solid rgba(109, 91, 208, 0.4);
        padding-bottom: 12px;
    }
    .modal-header-box h2 {
        color: #ffffff;
        font-size: 1.2rem;
        font-weight: 600;
        margin: 0;
    }
    .modal-icon {
        color: #8b5cf6;
        font-size: 1.2rem;
    }
    .form-group {
        display: flex;
        flex-direction: column;
        gap: 8px;
        margin-bottom: 20px;
        text-align: left;
    }
    .input-title {
        font-size: 0.65rem;
        color: #444c56;
        text-transform: uppercase;
        font-weight: bold;
        letter-spacing: 0.05em;
    }
    .modal-input {
        width: 100%;
        background: #150f24;
        border: 1px solid #6d5bd0;
        border-radius: 8px;
        padding: 12px;
        color: white;
        font-size: 0.9rem;
        outline: none;
        box-sizing: border-box;
    }
    .modal-input:focus {
        border-color: #8b5cf6;
        box-shadow: 0 0 0 3px rgba(139, 92, 246, 0.2);
    }
    .investment-plan-box {
        margin-top: 16px;
        border-top: 1px dashed #2d333b;
        padding-top: 16px;
    }
    .plan-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 16px;
    }
    .plan-header h3 {
        font-size: 0.85rem;
        color: #fff;
        margin: 0;
        font-weight: 600;
    }
    .percent-badge {
        font-size: 0.75rem;
        font-weight: bold;
        color: #f87171;
        background: rgba(248, 113, 113, 0.1);
        padding: 4px 8px;
        border-radius: 6px;
    }
    .percent-badge.valid {
        color: #8b5cf6;
        background: rgba(139, 92, 246, 0.15);
    }
    .sliders-list {
        display: flex;
        flex-direction: column;
        gap: 12px;
        max-height: 220px;
        overflow-y: auto;
        padding-right: 4px;
    }
    .slider-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 4px 0;
    }
    .new-group-row {
        background: rgba(139, 92, 246, 0.05);
        padding: 8px;
        border-radius: 8px;
        border: 1px dashed rgba(139, 92, 246, 0.25);
    }
    .g-name-label {
        font-size: 0.85rem;
        color: #94a3b8;
        text-align: left;
        max-width: 150px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    .slider-controls {
        display: flex;
        align-items: center;
        gap: 8px;
    }
    .neon-slider {
        accent-color: #8b5cf6;
        cursor: pointer;
        width: 130px;
    }
    .percent-val-input {
        width: 42px;
        background: #150f24;
        border: 1px solid #6d5bd0;
        color: #fff;
        padding: 4px 2px;
        border-radius: 6px;
        text-align: center;
        font-family: monospace;
        font-size: 0.85rem;
        -moz-appearance: textfield;
    }
    .percent-val-input:focus {
        border-color: #8b5cf6;
        box-shadow: 0 0 0 2px rgba(139, 92, 246, 0.2);
    }
    .percent-val-input::-webkit-outer-spin-button,
    .percent-val-input::-webkit-inner-spin-button {
        -webkit-appearance: none;
        margin: 0;
    }
    .unit-text {
        font-size: 0.85rem;
        color: #444c56;
        width: 10px;
    }
    .modal-actions {
        display: flex;
        gap: 12px;
        justify-content: flex-end;
        margin-top: 24px;
    }
    .btn-cancel {
        background: transparent;
        border: 1px solid #6d5bd0;
        color: #94a3b8;
        padding: 10px 16px;
        border-radius: 8px;
        cursor: pointer;
        font-weight: 600;
    }
    .btn-cancel:hover {
        background: #3d3560;
        color: #fff;
    }
    .btn-submit {
        background: #8b5cf6;
        border: none;
        color: #ffffff;
        padding: 10px 20px;
        border-radius: 8px;
        cursor: pointer;
        font-weight: 700;
    }
    .btn-submit:hover:not(:disabled) {
        background: #7c3aed;
    }
    .btn-submit:disabled {
        background: #3d3560;
        color: #7c6fb5;
        cursor: not-allowed;
        border: 1px solid #6d5bd0;
    }
</style>

