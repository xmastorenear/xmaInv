<script setup lang="ts">
    import { ref, onMounted } from 'vue';

    const emit = defineEmits<{
        (e: 'create', payload: { name: string }): void;
        (e: 'close'): void;
    }>();

    const groupName = ref('');
    const inputRef = ref<HTMLInputElement | null>(null);

    onMounted(() => {
        inputRef.value?.focus();
    });

    const handleSubmit = () => {
        if (groupName.value.trim()) {
            emit('create', { name: groupName.value.trim() });
            groupName.value = '';
        }
    };
</script>

<template>
    <div class="modal-card">
        <header>
            <div class="modal-icon">
                <i class="pi pi-box"></i>
            </div>
            <h2>{{ $t('asset.newGroup') }}</h2>
        </header>

        <div class="form-group">
            <label>{{ $t('asset.enterName') }}</label>
            <input
                ref="inputRef"
                v-model="groupName"
                type="text"
                :placeholder="$t('asset.placeholder')"
                @keyup.enter="handleSubmit"
                class="modal-input"
            >
        </div>

        <div class="modal-actions">
            <button class="btn-cancel" @click="emit('close')">
                {{ $t('common.cancel') }}
            </button>
            <button
                class="btn-submit"
                :disabled="!groupName.trim()"
                @click="handleSubmit"
            >
                {{ $t('common.create') }}
            </button>
        </div>
    </div>
</template>

<style scoped>
    header {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 12px;
        margin-bottom: 24px;
    }

    .modal-icon {
        width: 48px;
        height: 48px;
        background: rgba(0, 192, 135, 0.1);
        color: #00c087;
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 1.5rem;
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 8px;
        margin-bottom: 24px;
        text-align: left;
    }

    .form-group label {
        font-size: 0.7rem;
        color: #444c56;
        text-transform: uppercase;
        font-weight: bold;
        padding-left: 4px;
    }

    .modal-input {
        width: 100%;
        background: #0f1113;
        border: 1px solid #2d333b;
        border-radius: 10px;
        padding: 12px;
        color: white;
        font-size: 1rem;
        outline: none;
        box-sizing: border-box;
    }

    .modal-input:focus {
        border-color: #00c087;
    }
</style>
