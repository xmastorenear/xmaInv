<script setup lang="ts">
    import { ref, onMounted, nextTick, watch } from 'vue';
    import { useTBank } from '../../composables/useAppData/useTBank';
    import type { InstrumentCard } from '../../types';

    const props = defineProps<{
        groupId: number;
        presetTicker?: string;
        presetPrice?: number;
        isSell?: boolean;
        currentAmount?: number;
    }>();

    const emit = defineEmits<{
        (e: 'close'): void;
        (e: 'submit', payload: { group_id: number; ticker: string; amount: number; price: number; uid: string | null }): void;
    }>();

    const ticker = ref(props.presetTicker || '');
    const amount = ref<number | null>(null);
    const price = ref<number | null>(props.presetTicker ? (props.presetPrice ?? null) : null);

    const tickerInput = ref<HTMLInputElement | null>(null);
    const amountInput = ref<HTMLInputElement | null>(null);

    // ---------- Ticker search via the T-Bank API ----------
    const { searching, searchError, searchInstruments } = useTBank();
    const searchResults = ref<InstrumentCard[]>([]);
    const dropdownOpen = ref(false);
    const selectedUid = ref<string | null>(null);
    // Current ticker whose price has already been loaded (to avoid reloading on reopen/retype)
    const lastLoadedTicker = ref<string>('');

    let debounceTimer: ReturnType<typeof setTimeout> | null = null;

    const isPresetMode = () => !!props.presetTicker;

    const closeDropdown = () => {
        dropdownOpen.value = false;
    };

    const runSearch = async (query: string) => {
        const q = query.trim();
        if (!q || q.length < 2) {
            searchResults.value = [];
            dropdownOpen.value = false;
            return;
        }
        const results = await searchInstruments(q);
        searchResults.value = results;
        dropdownOpen.value = results.length > 0;
    };

    // Input debounce: search only on ticker change (not on reopen)
    const onTickerInput = () => {
        selectedUid.value = null;
        // If the ticker changed, reset the price (it will be fetched when selecting from the list)
        if (ticker.value.trim().toUpperCase() !== lastLoadedTicker.value) {
            if (!isPresetMode()) {
                price.value = null;
            }
        }
        if (debounceTimer) clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => runSearch(ticker.value), 350);
    };

    const selectInstrument = (card: InstrumentCard) => {
        ticker.value = card.ticker;
        selectedUid.value = card.uid;
        lastLoadedTicker.value = card.ticker;
        // Auto-fill the price from the API (remains editable)
        price.value = card.price ?? null;
        closeDropdown();
    };

    // Do not load the price for buy-more/sell mode (presetTicker is set)
    onMounted(async () => {
        await nextTick();
        if (props.presetTicker) {
            // Buy-more/sell mode: do NOT reload the price, use presetPrice
            amountInput.value?.focus();
        } else {
            tickerInput.value?.focus();
        }
    });

    watch(() => props.presetTicker, (val) => {
        if (val) {
            ticker.value = val;
            price.value = props.presetPrice ?? null;
            lastLoadedTicker.value = val;
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
            price: price.value,
            uid: selectedUid.value
        });
    };
</script>

<template>
    <div class="modal-card">
        <header class="modal-header-box">
            <div class="modal-icon-badge" :style="isSell ? 'background: rgba(248, 113, 113, 0.1); color: #f87171;' : ''">
                <i :class="isSell ? 'pi pi-percentage' : (presetTicker ? 'pi pi-shopping-bag' : 'pi pi-box')"></i>
            </div>
            <h2>{{ isSell ? `${$t('asset.sell')} ${presetTicker}` : (presetTicker ? `${$t('asset.buyMore')} ${presetTicker}` : $t('asset.addNew')) }}</h2>
        </header>

        <div class="form-grid">
            <!-- Ticker field with search -->
            <div class="form-group ticker-search-group">
                <label>{{ $t('asset.ticker') }}</label>
                <div class="ticker-search-box">
                    <input
                        ref="tickerInput"
                        v-model="ticker"
                        type="text"
                        :placeholder="$t('tbank.searchPlaceholder')"
                        class="modal-input"
                        :disabled="!!presetTicker"
                        @input="onTickerInput"
                        @focus="ticker.trim().length >= 2 && searchResults.length > 0 ? dropdownOpen = true : null"
                        @keydown.enter="handleSubmit"
                        @blur="closeDropdown"
                    />
                    <!-- Loading indicator -->
                    <span v-if="searching" class="search-spinner"></span>

                    <!-- Results dropdown -->
                    <div v-if="dropdownOpen && searchResults.length > 0" class="ticker-dropdown">
                        <div
                            v-for="card in searchResults"
                            :key="card.uid"
                            class="ticker-option"
                            @mousedown.prevent="selectInstrument(card)"
                        >
                            <div class="ticker-option-main">
                                <span class="ticker-option-ticker">{{ card.ticker }}</span>
                                <span class="ticker-option-name">{{ card.name }}</span>
                                <span class="ticker-option-class">{{ card.class_code }}</span>
                            </div>
                            <span class="ticker-option-price">{{ card.price !== null ? card.price.toFixed(2) : '—' }}</span>
                        </div>
                    </div>

                    <!-- Search error -->
                    <div v-if="searchError" class="ticker-error">{{ searchError }}</div>
                </div>
            </div>

            <div v-if="presetTicker && currentAmount !== undefined" class="balance-info-badge">
                <span class="balance-label">{{ $t('asset.currentBalance') }}:</span>
                <span class="balance-value">
                {{ formatAmount(currentAmount) }} <span class="balance-ticker-text">{{ presetTicker }}</span>
            </span>
            </div>

            <div class="form-group">
                <label>{{ $t('asset.amount') }}</label>
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
                <label>{{ $t('asset.pricePerUnit') }}</label>
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
            <button class="btn-cancel" @click="emit('close')">{{ $t('asset.cancel') }}</button>
            <button
                class="btn-submit"
                :class="{ 'btn-sell-style': isSell }"
                :disabled="!ticker.trim() || !amount || !price"
                @click="handleSubmit"
            >
                {{ isSell ? $t('asset.sell') : (presetTicker ? $t('asset.buyMore') : $t('asset.create')) }}
            </button>
        </div>
    </div>
</template>

<style scoped>
    .modal-card {
        background: #241f33;
        border: 1px solid #6d5bd0;
        padding: 30px;
        border-radius: 16px;
        width: 100%;
        max-width: 380px;
        box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
        text-align: center;
        z-index: 100000 !important;
    }

    .modal-header-box {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 12px;
        margin-bottom: 24px;
    }

    .modal-header-box h2 {
        color: white;
        margin: 0;
        font-size: 1.25rem;
    }

    .modal-icon-badge {
        width: 48px;
        height: 48px;
        border-radius: 50%;
        background: rgba(139, 92, 246, 0.12);
        color: #8b5cf6;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 1.3rem;
    }

    .form-grid {
        display: flex;
        flex-direction: column;
        gap: 16px;
        margin-bottom: 24px;
    }

    .form-group {
        text-align: left;
    }

    .form-group label {
        display: block;
        color: #94a3b8;
        font-size: 0.85rem;
        margin-bottom: 6px;
    }

    .modal-input {
        width: 100%;
        padding: 12px 14px;
        background: #150f24;
        border: 1px solid #3b2d63;
        border-radius: 10px;
        color: white;
        font-size: 1rem;
        outline: none;
        box-sizing: border-box;
        transition: border-color 0.2s, box-shadow 0.2s;
    }

    .modal-input:focus {
        border-color: #8b5cf6;
        box-shadow: 0 0 0 3px rgba(139, 92, 246, 0.2);
    }

    .modal-input:disabled {
        opacity: 0.55;
        cursor: not-allowed;
    }

    /* ---------- Ticker search ---------- */
    .ticker-search-box {
        position: relative;
    }

    .search-spinner {
        position: absolute;
        right: 12px;
        top: 50%;
        transform: translateY(-50%);
        width: 16px;
        height: 16px;
        border: 2px solid #6d5bd0;
        border-top-color: transparent;
        border-radius: 50%;
        animation: spin 0.7s linear infinite;
        pointer-events: none;
    }

    @keyframes spin {
        to { transform: translateY(-50%) rotate(360deg); }
    }

    .ticker-dropdown {
        position: absolute;
        top: calc(100% + 6px);
        left: 0;
        right: 0;
        background: #1a1d21;
        border: 1px solid #3b2d63;
        border-radius: 10px;
        max-height: 220px;
        overflow-y: auto;
        z-index: 10;
        box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
        text-align: left;
    }

    .ticker-option {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 10px;
        padding: 10px 12px;
        cursor: pointer;
        transition: background 0.15s;
    }

    .ticker-option:hover {
        background: #2d333b;
    }

    .ticker-option-main {
        display: flex;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
    }

    .ticker-option-ticker {
        color: #fff;
        font-weight: bold;
        font-size: 0.9rem;
    }

    .ticker-option-name {
        color: #94a3b8;
        font-size: 0.75rem;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 200px;
    }

    .ticker-option-price {
        color: #8b5cf6;
        font-weight: bold;
        font-size: 0.85rem;
        flex-shrink: 0;
    }

    .ticker-option-class {
        align-self: flex-start;
        padding: 1px 6px;
        margin-top: 2px;
        background: rgba(139, 92, 246, 0.12);
        border: 1px solid rgba(139, 92, 246, 0.3);
        border-radius: 4px;
        color: #a78bfa;
        font-size: 0.65rem;
        font-family: 'Courier New', monospace;
        letter-spacing: 0.03em;
        white-space: nowrap;
    }

    .ticker-error {
        margin-top: 8px;
        color: #f87171;
        font-size: 0.8rem;
        text-align: left;
        word-break: break-word;
    }

    /* ---------- Balance badge ---------- */
    .balance-info-badge {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 8px;
        padding: 10px 14px;
        background: rgba(139, 92, 246, 0.08);
        border: 1px solid rgba(139, 92, 246, 0.25);
        border-radius: 10px;
        font-size: 0.85rem;
    }

    .balance-label {
        color: #94a3b8;
    }

    .balance-value {
        color: #fff;
        font-weight: bold;
    }

    .balance-ticker-text {
        color: #8b5cf6;
        font-size: 0.75rem;
        margin-left: 4px;
    }

    /* ---------- Buttons ---------- */
    .modal-actions {
        display: flex;
        gap: 12px;
    }

    .btn-submit {
        flex: 1;
        background: #8b5cf6;
        color: white;
        border: none;
        padding: 12px;
        border-radius: 10px;
        font-weight: bold;
        cursor: pointer;
        transition: background 0.2s;
    }

    .btn-submit:hover {
        background: #7c3aed;
    }

    .btn-submit:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .btn-sell-style {
        background: #f87171;
    }

    .btn-sell-style:hover {
        background: #dc2626;
    }

    .btn-cancel {
        flex: 1;
        background: transparent;
        color: #94a3b8;
        border: 1px solid #6d5bd0;
        padding: 12px;
        border-radius: 10px;
        cursor: pointer;
        transition: background 0.2s;
    }

    .btn-cancel:hover {
        background: rgba(109, 91, 208, 0.1);
    }
</style>
