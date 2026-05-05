<script setup lang="ts">
    import { ref, onMounted } from 'vue';

    const props = defineProps<{
        sourceName: string;
        mode: 'deposit' | 'withdraw';
    }>();

    const emit = defineEmits<{
        (e: 'submit', payload: { amount: number, timestamp: string }): void;
        (e: 'close'): void;
    }>();

    const amount = ref<number | string>('');
    const timestamp = ref('');
    const inputRef = ref<HTMLInputElement | null>(null);

    onMounted(() => {
        const now = new Date();
        now.setMinutes(now.getMinutes() - now.getTimezoneOffset());
        timestamp.value = now.toISOString().slice(0, 16);

        inputRef.value?.focus();
    });

    const handleSubmit = () => {
        const value = Number(amount.value);
        if (value > 0 && timestamp.value) {
            const finalAmount = props.mode === 'deposit' ? value : -value;
            emit('submit', {
                amount: finalAmount,
                timestamp: timestamp.value.replace('T', ' ') + ':00'
            });
            amount.value = '';
        }
    };
</script>

<template>
    <div class="modal-card">
        <header>
            <div class="icon" :class="mode">
                {{ mode === 'deposit' ? '+' : '−' }}
            </div>
            <div class="title-group">
                <h2>{{ mode === 'deposit' ? $t('common.deposit') : $t('common.withdraw') }}</h2>
                <p>{{ sourceName }}</p>
            </div>
        </header>

        <div class="form-group">
            <div class="input-wrapper amount-input">
                <input
                    ref="inputRef"
                    v-model="amount"
                    type="number"
                    placeholder="0.00"
                    @keyup.enter="handleSubmit"
                >
                <span class="currency">₽</span>
            </div>

            <!-- Поле даты и времени -->
            <div class="datetime-wrapper">
                <label>{{ $t('source.date') }}</label>
                <input
                    v-model="timestamp"
                    type="datetime-local"
                    class="datetime-input"
                >
            </div>
        </div>

        <div class="actions">
            <button class="btn-cancel" @click="emit('close')">{{ $t('common.cancel') }}</button>
            <button
                class="btn-submit"
                :class="mode"
                :disabled="!amount || Number(amount) <= 0 || !timestamp"
                @click="handleSubmit"
            >
                {{ $t('common.confirm') }}
            </button>
        </div>
    </div>
</template>

<style scoped>
    input::-webkit-outer-spin-button,
    input::-webkit-inner-spin-button {
        -webkit-appearance: none;
        margin: 0;
    }

    input[type=number] {
        -moz-appearance: textfield;
    }

    .modal-card {
        background: #1a1d21;
        padding: 24px;
        border-radius: 16px;
        border: 1px solid #2d333b;
        width: 320px;
        box-shadow: 0 20px 40px rgba(0,0,0,0.4);
    }

    header { display: flex; align-items: center; gap: 16px; margin-bottom: 20px; }

    .icon { width: 40px; height: 40px; border-radius: 10px; display: flex; align-items: center; justify-content: center; font-size: 1.5rem; font-weight: bold; }

    .icon.deposit { background: rgba(0, 192, 135, 0.1); color: #00c087; }

    .icon.withdraw { background: rgba(248, 113, 113, 0.1); color: #f87171; }

    .title-group h2 { margin: 0; font-size: 1.2rem; color: white; }

    .title-group p { margin: 0; font-size: 0.85rem; color: #94a3b8; }

    .form-group { display: flex; flex-direction: column; gap: 16px; margin-bottom: 24px; }

    .input-wrapper {
        position: relative;
        margin-bottom: 24px;
    }

    .amount-input input {
        width: 100%;
        background: #0f1113;
        border: 1px solid #2d333b;
        border-radius: 12px;
        padding: 16px;
        font-size: 1.5rem;
        color: white;
        text-align: center;
        outline: none;
        box-sizing: border-box;
        appearance: none;
    }

    .amount-input input:focus + .currency,
    .amount-input input:hover + .currency {
        color: #444c56;
    }

    .currency {
        position: absolute;
        right: 20px;
        top: 50%;
        transform: translateY(-50%);
        color: #444c56;
        font-size: 1.2rem;
        pointer-events: none;
        user-select: none;
        z-index: 5;
        background: transparent !important;
    }

    .input-wrapper.amount-input {
        position: relative;
        background: transparent;
    }

    .datetime-wrapper { display: flex; flex-direction: column; gap: 6px; }

    .datetime-wrapper label { font-size: 0.7rem; color: #444c56; text-transform: uppercase; font-weight: bold; padding-left: 4px; }

    .datetime-input {
        background: #0f1113; border: 1px solid #2d333b; border-radius: 8px;
        padding: 10px; color: white; font-family: inherit; font-size: 0.9rem; outline: none;
    }
    .datetime-input::-webkit-calendar-picker-indicator { filter: invert(0.5); cursor: pointer; }

    .actions { display: flex; gap: 12px; }

    button { flex: 1; padding: 12px; border-radius: 10px; font-weight: bold; cursor: pointer; border: none; transition: 0.2s; }

    .btn-cancel { background: #2d333b; color: #94a3b8; }

    .btn-submit.deposit { background: #00c087; color: black; }

    .btn-submit.withdraw { background: #f87171; color: white; }

    button:disabled { opacity: 0.3; cursor: not-allowed; }
</style>
