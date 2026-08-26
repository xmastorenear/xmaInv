<script setup lang="ts">
    import { ref } from 'vue';

    const emit = defineEmits<{
        (e: 'create', payload: { name: string, color: string }): void
    }>();

    const inputName = ref('');

    const colors = ['#00c087', '#ff5f56', '#ffbd2e', '#4facfe', '#a166ab', '#f87171'];
    const selectedColor = ref(colors[0]);

    function submit() {
        if (inputName.value.trim()) {
            emit('create', {
                name: inputName.value.trim(),
                color: selectedColor.value
            });
            inputName.value = '';
        }
    }
</script>

<template>
    <div class="modal-card">
        <h2>{{ $t('strategy.newStrategy') }}</h2>
        <p class="subtitle">{{ $t('strategy.enterName') }}</p>

        <input
            v-model="inputName"
            :placeholder="$t('strategy.examplePlaceHolder')"
            @keyup.enter="submit"
            autofocus
        >

        <div class="color-picker">
            <div
                v-for="color in colors"
                :key="color"
                class="color-option"
                :style="{ backgroundColor: color }"
                :class="{ active: selectedColor === color }"
                @click="selectedColor = color"
            ></div>
        </div>

        <button @click="submit" :disabled="!inputName.trim()">
            {{ $t('strategy.createStrategy') }}
        </button>
    </div>
</template>

<style scoped>
    .modal-card {
        background: #241f33;
        padding: 2rem;
        border-radius: 16px;
        border: 1px solid #6d5bd0;
        width: 100%;
        max-width: 360px;
        text-align: center;
    }
    .modal-card h2, p {
        color: white;
    }

    .color-picker {
        display: flex;
        justify-content: center;
        gap: 10px;
        margin-bottom: 1.5rem;
    }

    .color-option {
        width: 24px;
        height: 24px;
        border-radius: 50%;
        cursor: pointer;
        border: 2px solid transparent;
        transition: transform 0.2s;
    }

    .color-option.active {
        border-color: white;
        transform: scale(1.2);
    }

    input {
        width: 100%;
        box-sizing: border-box;
        padding: 0.8rem;
        background: #150f24;
        border: 1px solid #6d5bd0;
        border-radius: 10px;
        color: white;
        margin-bottom: 1rem;
        outline: none;
        transition: border-color 0.2s, box-shadow 0.2s;
    }

    input:focus {
        border-color: #8b5cf6;
        box-shadow: 0 0 0 3px rgba(139, 92, 246, 0.2);
    }

    button {
        width: 100%;
        padding: 0.8rem;
        background: #8b5cf6;
        color: #ffffff;
        border: none;
        border-radius: 10px;
        font-weight: 700;
        cursor: pointer;
        transition: background 0.2s;
    }

    button:hover:not(:disabled) {
        background: #7c3aed;
    }

    button:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
</style>
