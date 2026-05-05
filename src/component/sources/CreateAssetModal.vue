<script setup lang="ts">
    import { ref, onMounted, nextTick } from 'vue';

    const props = defineProps<{
        groupId: number;
        presetTicker?: string;
        presetPrice?: number;
        isSell?: boolean;
        currentAmount?: number;
    }>();

    const emit = defineEmits<{
        (e: 'close'): void;
        (e: 'submit', payload: { group_id: number; ticker: string; amount: number; price: number }): void;
    }>();

    const ticker = ref(props.presetTicker || '');
    const amount = ref<number | null>(null);
    const price = ref<number | null>(props.presetTicker ? (props.presetPrice ?? null) : null);

    const tickerInput = ref<HTMLInputElement | null>(null);
    const amountInput = ref<HTMLInputElement | null>(null);

    onMounted(async () => {
        await nextTick();
        if (props.presetTicker) {
            amountInput.value?.focus();
        } else {
            tickerInput.value?.focus();
        }
    });
    const formatAmount = (val: number | undefined) => {
        if (val === undefined || val === null) return '0';
        return new Intl.NumberFormat('ru-RU', { maximumFractionDigits: 6 }).format(val);
    };
    const handleSubmit = () => {
        if (!ticker.value.trim() || !amount.value || !price.value) return;

        emit('submit', {
            group_id: props.groupId,
            ticker: ticker.value.trim().toUpperCase(),
            amount: amount.value,
            price: price.value
        });
    };
</script>

<template>
    <div class="modal-card">
        <header class="modal-header-box">
            <div class="modal-icon-badge" :style="isSell ? 'background: rgba(248, 113, 113, 0.1); color: #f87171;' : ''">
                <i :class="isSell ? 'pi pi-percentage' : (presetTicker ? 'pi pi-shopping-bag' : 'pi pi-box')"></i>
            </div>
            <h2>{{ isSell ? `Продать ${presetTicker}` : (presetTicker ? `Докупить ${presetTicker}` : 'Добавить новый active') }}</h2>
        </header>

        <div class="form-grid">
            <!-- Поле Тикер -->
            <div class="form-group">
                <label>Тикер / Название</label>
                <input
                    ref="tickerInput"
                    v-model="ticker"
                    type="text"
                    placeholder="BTC, AAPL, USD..."
                    class="modal-input"
                    :disabled="!!presetTicker"
                />
            </div>

            <div v-if="presetTicker && currentAmount !== undefined" class="balance-info-badge">
                <span class="balance-label">Текущий баланс:</span>
                <span class="balance-value">
                {{ formatAmount(currentAmount) }} <span class="balance-ticker-text">{{ presetTicker }}</span>
            </span>
            </div>

            <div class="form-group">
                <label>Количество</label>
                <input
                    ref="amountInput"
                    v-model.number="amount"
                    type="number"
                    step="any"
                    placeholder="0.00"
                    class="modal-input"
                />
            </div>

            <div class="form-group">
                <label>Цена за единицу (₽)</label>
                <input
                    v-model.number="price"
                    type="number"
                    step="any"
                    placeholder="0.00"
                    class="modal-input"
                    @keyup.enter="handleSubmit"
                />
            </div>
        </div>

        <div class="modal-actions">
            <button class="btn-cancel" @click="emit('close')">Отмена</button>
            <button
                class="btn-submit"
                :class="{ 'btn-sell-style': isSell }"
                :disabled="!ticker.trim() || !amount || !price"
                @click="handleSubmit"
            >
                {{ isSell ? 'Продать' : (presetTicker ? 'Докупить' : 'Создать') }}
            </button>
        </div>
    </div>
</template>

<style scoped>
    .modal-card {
        background: #0f1113;
        border: 1px solid #2d333b;
        border-radius: 12px;
        padding: 24px;
        width: 100%;
        max-width: 360px;
        box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
    }

    .modal-header-box {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 12px;
        margin-bottom: 24px;
    }

    .modal-header-box h2 {
        color: #ffffff;
        font-size: 1.2rem;
        font-weight: 600;
        margin: 0;
    }

    .modal-icon-badge {
        width: 48px;
        height: 48px;
        background: rgba(0, 192, 135, 0.1);
        color: #00c087;
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 1.4rem;
    }

    .form-grid {
        display: flex;
        flex-direction: column;
        gap: 16px;
        margin-bottom: 24px;
        text-align: left;
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .form-group label {
        font-size: 0.65rem;
        color: #444c56;
        text-transform: uppercase;
        font-weight: bold;
        letter-spacing: 0.05em;
        padding-left: 2px;
    }

    .modal-input {
        width: 100%;
        background: #0f1113;
        border: 1px solid #2d333b;
        border-radius: 10px;
        padding: 12px;
        color: white;
        font-size: 0.95rem;
        outline: none;
        box-sizing: border-box;
        transition: border-color 0.2s;
    }

    .modal-input:focus {
        border-color: #00c087;
    }

    .modal-input:disabled {
        opacity: 0.5;
        cursor: not-allowed;
        background: #141619;
        border-color: #23282e;
    }

    .modal-actions {
        display: flex;
        gap: 12px;
        justify-content: flex-end;
    }

    .btn-cancel {
        background: transparent;
        border: 1px solid #2d333b;
        color: #94a3b8;
        padding: 10px 16px;
        border-radius: 10px;
        cursor: pointer;
        font-weight: 600;
        font-size: 0.85rem;
        transition: all 0.2s;
    }

    .btn-cancel:hover {
        background: rgba(255, 255, 255, 0.02);
        color: #ffffff;
        border-color: #444c56;
    }

    .btn-submit {
        background: #00c087;
        border: none;
        color: #050505;
        padding: 10px 20px;
        border-radius: 10px;
        cursor: pointer;
        font-weight: 700;
        font-size: 0.85rem;
        transition: all 0.2s;
    }

    .btn-submit:hover:not(:disabled) {
        background: #00e09e;
        box-shadow: 0 0 12px rgba(0, 192, 135, 0.4);
    }

    .btn-submit:disabled {
        background: #1c2128;
        color: #444c56;
        cursor: not-allowed;
        border: 1px solid #2d333b;
    }

    .btn-sell-style {
        background: #f87171 !important;
        color: #050505 !important;
    }

    .btn-sell-style:hover:not(:disabled) {
        background: #fca5a5 !important;
        box-shadow: 0 0 12px rgba(248, 113, 113, 0.4) !important;
    }

    /* Стили информационной плашки баланса */
    .balance-info-badge {
        background: #141619;
        border: 1px dashed #2d333b;
        border-radius: 10px;
        padding: 10px 14px;
        margin-bottom: 20px;
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-size: 0.8rem;
    }

    .balance-label {
        color: #444c56;
        text-transform: uppercase;
        font-weight: 700;
        letter-spacing: 0.05em;
        font-size: 0.65rem;
    }

    .balance-value {
        color: #ffffff;
        font-weight: 700;
        font-family: 'Courier New', monospace;
    }

    .balance-ticker-text {
        color: #00c087;
        font-size: 0.75rem;
        font-weight: bold;
    }
</style>
