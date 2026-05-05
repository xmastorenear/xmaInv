<script setup lang="ts">
    import { ref } from 'vue';

    const emit = defineEmits<{
        (e: 'create', payload: { name: string }): void;
        (e: 'close'): void;
    }>();

    const inputName = ref('');
    function submit() {
        if (inputName.value.trim()) {
            emit('create', { name: inputName.value.trim() });
            inputName.value = '';
        }
    }
</script>

<template>
    <div class="modal-card">
        <h2>{{ $t('source.newSource') }}</h2>
        <p class="subtitle">{{ $t('source.enterName') }}</p>

        <input
            v-model="inputName"
            :placeholder="$t('source.placeholder')"
            @keyup.enter="submit"
            autofocus
        >

        <div class="modal-actions">
            <button class="btn-cancel" @click="emit('close')">
                {{ $t('common.cancel') }}
            </button>
            <button class="btn-submit" @click="submit" :disabled="!inputName.trim()">
                {{ $t('common.create') }}
            </button>
        </div>
    </div>
</template>

<style scoped>
    .modal-card {
        background: #1a1d21;
        padding: 2rem;
        border-radius: 16px;
        border: 1px solid #2d333b;
        width: 100%;
        max-width: 360px;
        text-align: center;
    }

    h2 { color: white; margin-bottom: 8px; }

    .subtitle { color: #94a3b8; font-size: 0.9rem; margin-bottom: 24px; }

    input {
        width: 100%;
        padding: 12px;
        background: #0f1113;
        border: 1px solid #2d333b;
        border-radius: 8px;
        color: white;
        margin-bottom: 20px;
        box-sizing: border-box;
    }

    .modal-actions { display: flex; gap: 12px; }

    button {
        flex: 1;
        padding: 12px;
        border-radius: 8px;
        font-weight: 600;
        cursor: pointer;
        transition: 0.2s;
    }

    .btn-submit { background: #00c087; color: black; border: none; }

    .btn-submit:disabled { opacity: 0.5; }

    .btn-cancel { background: transparent; color: #94a3b8; border: 1px solid #2d333b; }

    .btn-cancel:hover { background: #2d333b; }
</style>
