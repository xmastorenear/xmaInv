<script setup lang="ts">
    interface Source {
        id: number;
        strategy_id: number;
        name: string;
        icon_url: string;
        total_balance: number;
        profit_loss: number;
    }

    const props = defineProps<{
        source: Source
    }>();

    const emit = defineEmits<{
        (e: 'deposit'): void;
        (e: 'withdraw'): void;
        (e: 'contextmenu', event: MouseEvent): void;
    }>();
    const formatCurrency = (val: number) => {
        return new Intl.NumberFormat('ru-RU').format(val) + ' ₽';
    };
    const handleDeposit = () => emit('deposit');
    const handleWithdraw = () => emit('withdraw');
</script>


<template>
    <div class="source-card" @contextmenu.prevent="emit('contextmenu', $event)">
        <div class="action-btn deposit" @click.stop="handleDeposit">+</div>
        <div class="action-btn withdraw" @click.stop="handleWithdraw">−</div>

        <div class="card-content">
            <div class="card-row header-row-sc">
                <div class="source-icon">
                    <img :src="source.icon_url" v-if="source.icon_url">
                    <div class="icon-placeholder" v-else>
                        {{ props.source.name.charAt(0).toUpperCase() }}
                    </div>
                </div>
                <div class="source-name" :title="source.name">
                    {{ source.name }}
                </div>
            </div>

            <div class="card-row data-row">
                <div class="balance-box">
                    <span class="label">{{ $t('source.balance') }}:</span>
                    <span class="value">{{ formatCurrency(source.total_balance) }}</span>
                </div>
                <div class="pnl-box" :class="source.profit_loss >= 0 ? 'positive' : 'negative'">
                    {{ source.profit_loss >= 0 ? '+' : '' }}{{ formatCurrency(source.profit_loss) }}
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
    .source-card {
        position: relative;
        overflow: hidden;
        max-width: 15vw; /* Увеличил, чтобы текст влезал */
        min-width: 180px;
        background: #1a1d21;
        border: 1px solid #2d333b;
        border-radius: 8px;
        padding: 10px 12px;
        display: flex;
        flex-direction: column;
        gap: 8px;
        flex-shrink: 0;
        transition: border-color 0.2s ease;
        height: 75px;
    }

    .source-card:hover {
        border-color: #444c56;
    }

    .source-card:hover .action-btn {
        opacity: 1;
    }

    .source-card:hover .card-content {
        filter: blur(1px);
        opacity: 0.3;
    }

    .card-content {
        transition: all 0.2s ease;
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .card-row {
        display: flex;
        align-items: center;
        width: 100%;
    }

    .header-row-sc {
        gap: 10px;
        border-bottom: 1px solid #2d333b;
        padding-bottom: 6px;
        height: 45px;
    }

    .source-icon {
        width: 24px;
        height: 24px;
        border-radius: 4px;
        background: #2d333b;
        flex-shrink: 0;
        overflow: hidden;
    }

    .source-icon img { width: 100%; height: 100%; object-fit: cover; }

    .icon-placeholder {
        width: 100%; height: 100%;
        display: flex; align-items: center; justify-content: center;
        color: #00c087; font-weight: bold; font-size: 0.75rem;
    }

    .source-name {
        font-size: 0.85rem;
        color: #fff;
        font-weight: 500;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .data-row {
        justify-content: space-between;
        font-size: 0.75rem;
    }

    .balance-box {
        display: flex;
        gap: 4px;
        align-items: center;
    }

    .balance-box .label {
        color: #444c56;
        text-transform: uppercase;
        font-size: 0.65rem;
    }

    .balance-box .value {
        color: #94a3b8;
        font-weight: 600;
        font-size: 0.9rem;
    }

    .pnl-box { font-weight: 700; }

    .action-btn {
        position: absolute;
        top: 0;
        width: 50px;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background: rgba(0, 192, 135, 0.15);
        color: #00c087;
        font-size: 1.4rem;
        font-weight: bold;
        cursor: pointer;
        opacity: 0;
        transition: all 0.2s ease;
        z-index: 10;
    }

    .deposit { left: 0; border-right: 1px solid rgba(0, 192, 135, 0.2); }

    .withdraw {
        right: 0;
        color: #f87171;
        background: rgba(248, 113, 113, 0.15);
        border-left: 1px solid rgba(248, 113, 113, 0.2);
    }

    .action-btn:hover { background: #00c087; color: #000; }

    .withdraw:hover { background: #f87171; color: #000; }

    .positive { color: #00c087; }

    .negative { color: #f87171; }
</style>
