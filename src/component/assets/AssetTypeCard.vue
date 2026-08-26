<script setup lang="ts">
    import { ref, computed, onMounted, onBeforeUnmount, watch, nextTick } from 'vue';
    import type { Asset, AssetGroup } from "../../types";
    import CreateAssetModal from "../sources/CreateAssetModal.vue";

    const props = defineProps<{
        asset: AssetGroup,
        isExpanded: boolean,
        allAssets: Asset[],
        marketPrices?: Record<number, { price: number; stale: boolean }>,
        pricesLoading?: boolean
    }>();

    const emit = defineEmits<{
        (e: 'expand', id: number): void;
        (e: 'delete', id: number): void;
        (e: 'add-asset', id: number): void;
        (e: 'delete-asset', assetId: number): void;
        (e: 'buy-more', payload: { asset_id: number; amount: number; price: number }): void;
        (e: 'add-new-asset', payload: any): void;
        (e: 'sell-asset', payload: { asset_id: number; amount: number; price: number }): void;
        (e: 'refresh-prices'): void;
    }>();


    const bubbleMenuVisible = ref(false);
    const bubbleMenuX = ref(0);
    const bubbleMenuY = ref(0);
    const activeAssetIdForOp = ref<number | null>(null);

    const showAssetModal = ref(false);
    const isBuyMoreMode = ref(false);

    const containerRef = ref<HTMLDivElement | null>(null);
    const realWidth = ref(400);
    const realHeight = ref(350);

    const showSellModal = ref(false);
    interface PhysicalBubble extends Asset {
        size: number;
        targetSize: number;
        x: number;
        y: number;
        vx: number;
        vy: number;
        style: any;
        showText: boolean;
    }

    const physicsBubbles = ref<PhysicalBubble[]>([]);
    let resizeObserver: ResizeObserver | null = null;
    let animationFrameId: number | null = null;

    const paddingBetween = 12;
    const paddingFromEdges = 22;

    const selectedAssetForBuy = computed(() => {
        return physicsBubbles.value.find(b => b.id === activeAssetIdForOp.value) || null;
    });

    const currentGroupAssets = computed(() => {
        if (!props.allAssets) return [];
        return props.allAssets.filter(a => Number(a.group_id) === Number(props.asset.id));
    });

    const totalGroupValue = computed(() => {
        return currentGroupAssets.value.reduce((sum, item) => sum + (item.amount * item.buy_price), 0);
    });
    const openSellModal = () => {
        bubbleMenuVisible.value = false;
        if (!selectedAssetForBuy.value) return;
        showSellModal.value = true;
    };
    const openCreateAssetModal = () => {
        isBuyMoreMode.value = false;
        activeAssetIdForOp.value = null;
        showAssetModal.value = true;
    };
    const openBuyMoreModal = () => {
        bubbleMenuVisible.value = false;
        if (!selectedAssetForBuy.value) return;
        isBuyMoreMode.value = true;
        showAssetModal.value = true;
    };
    const handleSellSubmit = (payload: { group_id: number; ticker: string; amount: number; price: number }) => {
        showSellModal.value = false;
        if (selectedAssetForBuy.value) {
            emit('sell-asset', {
                asset_id: selectedAssetForBuy.value.id,
                amount: payload.amount,
                price: payload.price
            });
        }
    };
    const handleAssetSubmit = (payload: { group_id: number; ticker: string; amount: number; price: number }) => {
        showAssetModal.value = false;
        showSellModal.value = false;

        if ((isBuyMoreMode.value || showSellModal.value) && selectedAssetForBuy.value) {
            if (showSellModal.value) {
                emit('sell-asset', {
                    asset_id: selectedAssetForBuy.value.id,
                    amount: payload.amount,
                    price: payload.price
                });
            } else {
                emit('buy-more', {
                    asset_id: selectedAssetForBuy.value.id,
                    amount: payload.amount,
                    price: payload.price
                });
            }
        } else {
            emit('add-new-asset', payload);
        }
    };
    const relaxPositions = (items: PhysicalBubble[], width: number, height: number, steps: number) => {
        const friction = 0.78;
        const wallRepulsion = 0.35;

        for (let step = 0; step < steps; step++) {
            for (const c of items) {
                c.vx += (Math.random() - 0.5) * 0.008;
                c.vy += (Math.random() - 0.5) * 0.008;
            }

            for (let j = 0; j < items.length; j++) {
                for (let k = j + 1; k < items.length; k++) {
                    const c1 = items[j];
                    const c2 = items[k];
                    const dx = c2.x - c1.x;
                    const dy = c2.y - c1.y;
                    let distance = Math.hypot(dx, dy);
                    const minDistance = (c1.size / 2) + (c2.size / 2) + paddingBetween;

                    if (distance < minDistance) {
                        if (distance < 0.1) {
                            const randAngle = Math.random() * Math.PI * 2;
                            c2.x += Math.cos(randAngle) * 2;
                            c2.y += Math.sin(randAngle) * 2;
                            distance = 2;
                        }

                        const overlap = minDistance - distance;
                        const nx = dx / distance;
                        const ny = dy / distance;

                        const m1 = Math.pow(c1.size / 2, 2);
                        const m2 = Math.pow(c2.size / 2, 2);
                        const totalMass = m1 + m2;

                        const force = overlap * 0.65;
                        const impulseX = nx * force;
                        const impulseY = ny * force;

                        c1.vx -= impulseX * (m2 / totalMass);
                        c1.vy -= impulseY * (m2 / totalMass);
                        c2.vx += impulseX * (m1 / totalMass);
                        c2.vy += impulseY * (m1 / totalMass);
                    }
                }
            }

            for (const c of items) {
                const radius = c.size / 2;
                const distToLeft = c.x - paddingFromEdges;
                const distToRight = width - paddingFromEdges - c.x;
                const distToTop = c.y - paddingFromEdges;
                const distToBottom = height - paddingFromEdges - c.y;

                if (distToLeft < radius) c.vx += (radius - distToLeft) * wallRepulsion;
                else if (distToRight < radius) c.vx -= (radius - distToRight) * wallRepulsion;

                if (distToTop < radius) c.vy += (radius - distToTop) * wallRepulsion;
                else if (distToBottom < radius) c.vy -= (radius - distToBottom) * wallRepulsion;

                c.x += c.vx;
                c.y += c.vy;
                c.vx *= friction;
                c.vy *= friction;

                const minX = radius + paddingFromEdges;
                const maxX = width - radius - paddingFromEdges;
                const minY = radius + paddingFromEdges;
                const maxY = height - radius - paddingFromEdges;

                if (c.x < minX) { c.x = minX; c.vx = 0; }
                else if (c.x > maxX) { c.x = maxX; c.vx = 0; }
                if (c.y < minY) { c.y = minY; c.vy = 0; }
                else if (c.y > maxY) { c.y = maxY; c.vy = 0; }
            }
        }
    };
    const calculateTargetSizes = (assetsList: Asset[]): Record<number, number> => {
        const sizes: Record<number, number> = {};
        if (!assetsList.length) return sizes;

        const totalValue = assetsList.reduce((sum, item) => sum + (item.amount * item.buy_price), 0);

        if (totalValue === 0) {
            assetsList.forEach(item => sizes[item.id] = props.isExpanded ? 80 : 45);
            return sizes;
        }

        const weights: Record<number, number> = {};
        assetsList.forEach(item => {
            const itemValue = item.amount * item.buy_price;
            const share = itemValue / totalValue;
            weights[item.id] = Math.sqrt(share);
        });

        let currentWidth = realWidth.value > 100 ? realWidth.value : (props.isExpanded ? 1200 : 300);
        let currentHeight = realHeight.value > 100 ? realHeight.value : (props.isExpanded ? 800 : 350);
        const containerArea = currentWidth * currentHeight;

        const fillRatio = props.isExpanded ? 0.52 : 0.33;
        const targetTotalArea = containerArea * fillRatio;

        let sumWeightSquares = 0;
        assetsList.forEach(item => {
            sumWeightSquares += Math.pow(weights[item.id], 2);
        });

        let scale = Math.sqrt(targetTotalArea / (Math.PI * sumWeightSquares));

        if (assetsList.length < 5) {
            scale *= (1 + (5 - assetsList.length) * 0.15);
        }

        const minPx = props.isExpanded ? 70 : 40;
        const maxPx = assetsList.length === 1
            ? (props.isExpanded ? currentHeight * 0.75 : currentHeight * 0.65)
            : (props.isExpanded ? currentHeight * 0.52 : currentHeight * 0.42);

        assetsList.forEach(item => {
            const itemValue = item.amount * item.buy_price;
            const share = itemValue / totalValue;
            const weight = Math.sqrt(share);
            let calculatedDiameter = weight * scale * 2;

            if (calculatedDiameter < minPx) calculatedDiameter = minPx;
            if (calculatedDiameter > maxPx) calculatedDiameter = maxPx;
            sizes[item.id] = Math.round(calculatedDiameter);
        });
        return sizes;
    };
    const openBubbleMenu = async (e: MouseEvent, assetId: number) => {
        e.preventDefault();
        e.stopPropagation();
        activeAssetIdForOp.value = assetId;
        bubbleMenuX.value = e.clientX;
        bubbleMenuY.value = e.clientY;
        bubbleMenuVisible.value = true;

        await nextTick();
        const closeMenu = () => {
            bubbleMenuVisible.value = false;
            window.removeEventListener('click', closeMenu);
        };
        window.addEventListener('click', closeMenu);
    };
    const triggerDeleteAsset = () => {
        if (activeAssetIdForOp.value !== null) {
            emit('delete-asset', activeAssetIdForOp.value);
            bubbleMenuVisible.value = false;
        }
    };

    watch(currentGroupAssets, (newAssets) => {
        const updatedBubbles = physicsBubbles.value.filter(b => newAssets.some(a => a.id === b.id));
        const baseSizes = calculateTargetSizes(newAssets);

        newAssets.forEach(asset => {
            const existingBubble = updatedBubbles.find(b => b.id === asset.id);
            const targetSize = baseSizes[asset.id] || 40;
            if (existingBubble) {
                existingBubble.targetSize = targetSize;
                existingBubble.amount = asset.amount;
                existingBubble.buy_price = asset.buy_price;
            } else {
                const angle = Math.random() * Math.PI * 2;
                const spawnRadius = 20 + Math.random() * 30;
                const pushForce = 1.5;
                updatedBubbles.push({
                    ...asset,
                    size: 0,
                    targetSize: targetSize,
                    x: (realWidth.value / 2) + Math.cos(angle) * spawnRadius,
                    y: (realHeight.value / 2) + Math.sin(angle) * spawnRadius,
                    vx: Math.cos(angle) * pushForce,
                    vy: Math.sin(angle) * pushForce,
                    style: {},
                    showText: false
                });
            }
        });
        physicsBubbles.value = updatedBubbles;
    }, { deep: true, immediate: true });

    onMounted(() => {
        if (containerRef.value) {
            realWidth.value = containerRef.value.clientWidth || 400;
            realHeight.value = containerRef.value.clientHeight || 350;

            resizeObserver = new ResizeObserver((entries) => {
                for (let entry of entries) {
                    const { width, height } = entry.contentRect;
                    if (width > 0 && height > 0) {
                        realWidth.value = width;
                        realHeight.value = height;
                        const baseSizes = calculateTargetSizes(currentGroupAssets.value);
                        physicsBubbles.value.forEach(b => {
                            if (baseSizes[b.id]) b.targetSize = baseSizes[b.id];
                        });
                    }
                }
            });
            resizeObserver.observe(containerRef.value);
        }
        const tick = () => {
            const items = physicsBubbles.value;
            if (items.length > 0) {
                for (const c of items) {
                    if (c.targetSize !== undefined && c.size !== c.targetSize) {
                        c.size += (c.targetSize - c.size) * 0.04;
                        if (Math.abs(c.size - c.targetSize) < 0.1) {
                            c.size = c.targetSize;
                        }
                    }
                }
                relaxPositions(items, realWidth.value, realHeight.value, 5);
                for (const c of items) {
                    c.style = {
                        width: c.size + 'px',
                        height: c.size + 'px',
                        left: (c.x - c.size / 2) + 'px',
                        top: (c.y - c.size / 2) + 'px',
                        position: 'absolute' as const
                    };
                }
                for (const c of items) {
                    c.showText = bubbleTextFits(c);
                }
                physicsBubbles.value = [...items];
            }
            animationFrameId = requestAnimationFrame(tick);
        };
        animationFrameId = requestAnimationFrame(tick);
    });

    onBeforeUnmount(() => {
        if (resizeObserver) resizeObserver.disconnect();
        if (animationFrameId) cancelAnimationFrame(animationFrameId);
    });

    function formatCurrency(val: number) {
        return new Intl.NumberFormat('ru-RU').format(val) + ' ₽';
    }

    interface MarketInfo {
        marketValue: number;
        costValue: number;
        outer: number;
        inner: number;
        deltaPct: number;
        hasPrice: boolean;
    }

    const marketInfoMap = computed<Record<number, MarketInfo>>(() => {
        const map: Record<number, MarketInfo> = {};
        for (const item of physicsBubbles.value) {
            const costValue = item.amount * item.buy_price;
            const mkt = props.marketPrices?.[item.id];
            if (mkt && typeof mkt.price === 'number' && mkt.price > 0) {
                const marketValue = mkt.price * item.amount;
                map[item.id] = {
                    marketValue,
                    costValue,
                    outer: Math.max(marketValue, costValue),
                    inner: Math.min(marketValue, costValue),
                    deltaPct: costValue > 0 ? ((marketValue - costValue) / costValue) * 100 : 0,
                    hasPrice: true
                };
            } else {
                map[item.id] = {
                    marketValue: costValue,
                    costValue,
                    outer: costValue,
                    inner: costValue,
                    deltaPct: 0,
                    hasPrice: false
                };
            }
        }
        return map;
    });

    watch(() => props.marketPrices, () => {
        const baseSizes = calculateTargetSizes(currentGroupAssets.value);
        physicsBubbles.value.forEach(b => {
            const info = marketInfoMap.value[b.id];
            const base = baseSizes[b.id] || 40;
            if (info && info.outer > 0 && info.costValue > 0) {
                b.targetSize = base * Math.sqrt(info.outer / info.costValue);
            } else {
                b.targetSize = base;
            }
        });
    });

    const frameClass = (item: PhysicalBubble): string => {
        const info = marketInfoMap.value[item.id];
        if (!info || !info.hasPrice) return 'no-price';
        return info.deltaPct >= 0 ? 'profit' : 'loss';
    };

    const innerPct = (item: PhysicalBubble): number => {
        const info = marketInfoMap.value[item.id];
        if (!info || info.outer <= 0) return 100;
        return Math.round((info.inner / info.outer) * 100);
    };

    const deltaText = (item: PhysicalBubble): string => {
        const info = marketInfoMap.value[item.id];
        if (!info || !info.hasPrice) return '';
        return `${info.deltaPct >= 0 ? '+' : ''}${info.deltaPct.toFixed(1)}%`;
    };

    const bubbleValueText = (item: PhysicalBubble): string => {
        const info = marketInfoMap.value[item.id];
        if (info && info.hasPrice) return formatCurrency(info.marketValue);
        return formatCurrency(item.amount * item.buy_price);
    };

    const bubbleTitle = (item: PhysicalBubble): string => {
        const info = marketInfoMap.value[item.id];
        const costStr = formatCurrency(item.amount * item.buy_price);
        if (!info || !info.hasPrice) {
            return `${item.ticker}: ${costStr} (нет рыночной цены)`;
        }
        const delta = `${info.deltaPct >= 0 ? '+' : ''}${info.deltaPct.toFixed(1)}%`;
        return `${item.ticker} | ${formatCurrency(info.marketValue)} | Δ ${delta} (от ${costStr})`;
    };

    const MIN_TEXT_DIAMETER = 58;

    const bubbleFitsText = (item: PhysicalBubble): boolean => {
        return item.showText;
    };

    const measureCtx = (() => {
        const c = document.createElement('canvas');
        return c.getContext('2d');
    })();
    const textWidth = (text: string, bold: boolean, fontSizePx: number): number => {
        if (!measureCtx) return text.length * fontSizePx * 0.6;
        measureCtx.font = `${bold ? '700' : '400'} ${fontSizePx}px 'Segoe UI', 'Trebuchet MS', sans-serif`;
        return measureCtx.measureText(text).width;
    };
    const clampPx = (minRem: number, vw: number, maxRem: number): number => {
        const px = 16;
        const min = minRem * px;
        const max = maxRem * px;
        return Math.min(max, Math.max(min, (vw / 100) * window.innerWidth));
    };
    const fontSizes = () => ({
        tickerPx: clampPx(0.6, 2.5, 1.3),
        valuePx: clampPx(0.6, 2.0, 1.1),
        percentPx: clampPx(0.55, 1.8, 1.0),
    });
    const rowPad = (fontPx: number): number => (fontPx === fontSizes().percentPx ? 0 : 16);
    const lineHeight = (fontPx: number): number => Math.round(fontPx * 1.2);
    const bubbleTextFits = (item: PhysicalBubble): boolean => {
        const d = Math.min(item.size, item.targetSize ?? item.size);
        if (d < MIN_TEXT_DIAMETER) return false;
        const r = d / 2;
        const { tickerPx, valuePx, percentPx } = fontSizes();
        const value = bubbleValueText(item);
        const delta = deltaText(item);
        const gap = 4; // .bubble-text gap
        const rows: Array<{ text: string; fontPx: number }> = [
            { text: item.ticker, fontPx: tickerPx },
            { text: value, fontPx: valuePx },
        ];
        if (delta) rows.push({ text: delta, fontPx: percentPx });
        const blockH = rows.reduce((a, row) => a + lineHeight(row.fontPx), 0) + gap * (rows.length - 1);
        let top = -blockH / 2;
        for (const row of rows) {
            const h = lineHeight(row.fontPx);
            const centerOff = top + h / 2;
            const edgeOff = Math.abs(centerOff) + h / 2;
            const halfChord = Math.sqrt(Math.max(0, r * r - edgeOff * edgeOff));
            const avail = 2 * halfChord - 28;
            const need = textWidth(row.text, true, row.fontPx) + rowPad(row.fontPx);
            if (need > avail) return false;
            top += h + gap;
        }
        return true;
    };

    const tooltipBubble = ref<PhysicalBubble | null>(null);
    const tooltipX = ref(0);
    const tooltipY = ref(0);

    const showBubbleTooltip = (e: MouseEvent, item: PhysicalBubble) => {
        tooltipBubble.value = item;
        tooltipX.value = e.clientX;
        tooltipY.value = e.clientY;
    };
    const hideBubbleTooltip = () => {
        tooltipBubble.value = null;
    };
</script>

<template>
    <div class="asset-type-card" :class="{ 'expanded': isExpanded }">
        <Teleport to="body">
            <Transition name="fade-scale">
                <div v-if="bubbleMenuVisible"
                     class="bubble-context-menu"
                     :style="{ top: bubbleMenuY + 'px', left: bubbleMenuX + 'px' }">
                    <div class="menu-item" @click="openBuyMoreModal">
                        <i class="pi pi-plus-circle" style="color: #00c087;"></i>
                        <span>Докупить</span>
                    </div>
                    <div class="menu-item" @click="openSellModal">
                        <i class="pi pi-minus-circle" style="color: #f87171;"></i>
                        <span>Продать</span>
                    </div>
                    <div class="menu-divider"></div>
                    <div class="menu-item delete" @click="triggerDeleteAsset">
                        <i class="pi pi-trash"></i>
                        <span>Удалить актив {{ asset.name }}</span>
                    </div>
                </div>
            </Transition>
        </Teleport>

        <div class="asset-header">
            <span class="asset-label">{{ asset.name }}</span>
            <div class="asset-header-menu">
                <button class="action-btn add-item-btn" @click.stop="openCreateAssetModal">
                    <i class="pi pi-plus-circle"></i>
                </button>
                <button class="action-btn delete-btn" @click.stop="emit('delete', asset.id)">
                    <i class="pi pi-trash"></i>
                </button>
                <button class="action-btn expand-btn" @click.stop="emit('expand', asset.id)">
                    <i :class="isExpanded ? 'pi pi-times' : 'pi pi-expand'"></i>
                </button>
            </div>
        </div>

        <div class="asset-body">
            <div class="asset-scale"></div>
            <div class="asset-container" ref="containerRef">
                <div class="grid-overlay"></div>
                <div v-if="physicsBubbles.length > 0" class="assets-bubbles-stage">
                    <div v-if="pricesLoading" class="prices-spinner">
                        <i class="pi pi-spin pi-spinner"></i>
                    </div>
                    <div
                        v-for="item in physicsBubbles"
                        :key="item.id"
                        class="asset-bubble"
                        :class="frameClass(item)"
                        :style="item.style"
                        @mouseenter="e => showBubbleTooltip(e, item)"
                        @mousemove="e => showBubbleTooltip(e, item)"
                        @mouseleave="hideBubbleTooltip"
                        @contextmenu.prevent.stop="e => openBubbleMenu(e, item.id)"
                        @click.stop="emit('refresh-prices')"
                    >
                        <div
                            class="bubble-core"
                            :style="{ width: innerPct(item) + '%', height: innerPct(item) + '%' }"
                        ></div>
                        <div v-if="bubbleFitsText(item)" class="bubble-text">
                            <span class="bubble-ticker">{{ item.ticker }}</span>
                            <span class="bubble-value">{{ bubbleValueText(item) }}</span>
                            <span v-if="deltaText(item)" class="bubble-percent" :class="frameClass(item)">
                                {{ deltaText(item) }}
                            </span>
                        </div>
                    </div>
                </div>
                <Teleport to="body">
                    <div v-if="tooltipBubble" class="bubble-tooltip" :style="{ top: tooltipY + 'px', left: tooltipX + 'px' }">
                        <div class="tooltip-ticker">{{ tooltipBubble.ticker }}</div>
                        <div class="tooltip-value">{{ bubbleValueText(tooltipBubble) }}</div>
                        <div v-if="deltaText(tooltipBubble)" class="tooltip-percent" :class="frameClass(tooltipBubble)">
                            {{ deltaText(tooltipBubble) }}
                        </div>
                        <div class="tooltip-cost">{{ bubbleTitle(tooltipBubble) }}</div>
                    </div>
                </Teleport>
                <div class="content-placeholder" v-if="!isExpanded && physicsBubbles.length === 0">
                </div>
                <div class="content-placeholder" v-if="!isExpanded && physicsBubbles.length === 0">
                    <span>{{ asset.name }}</span>
                </div>
                <div class="expanded-content" v-else-if="isExpanded && physicsBubbles.length === 0">
                    <h3>Детальный просмотр активов</h3>
                </div>
            </div>
        </div>

        <div class="asset-footer">
            <div class="footer-info">
                <span class="footer-label">Market Value</span>
                <span class="asset-value">{{ formatCurrency(totalGroupValue) }}</span>
            </div>
        </div>
    </div>

    <Teleport to="body">
        <Transition name="modal-fade">
            <div v-if="showAssetModal" class="modal-overlay asset-creation-layer" @click.self="showAssetModal = false">
                <CreateAssetModal
                    :group-id="asset.id"
                    :preset-ticker="isBuyMoreMode && selectedAssetForBuy ? selectedAssetForBuy.ticker : undefined"
                    :preset-price="isBuyMoreMode && selectedAssetForBuy ? selectedAssetForBuy.buy_price : undefined"
                    :is-sell="showSellModal"
                    :current-amount="selectedAssetForBuy ? selectedAssetForBuy.amount : undefined"
                    @close="showAssetModal = false; showSellModal = false;"
                    @submit="handleAssetSubmit"
                />
            </div>
        </Transition>
    </Teleport>
    <Transition name="modal-fade">
        <div v-if="showSellModal" class="modal-overlay asset-creation-layer" @click.self="showSellModal = false">
            <CreateAssetModal
                :group-id="asset.id"
                :preset-ticker="selectedAssetForBuy?.ticker"
                :preset-price="selectedAssetForBuy?.buy_price"
                :is-sell="true"
                @close="showSellModal = false"
                @submit="handleSellSubmit"
            />
        </div>
    </Transition>
</template>
<style scoped>
    .asset-type-card {
        background: #0f1113;
        border: 1px solid #2d333b;
        border-radius: 12px;
        display: flex;
        flex-direction: column;
        width: 22vw;
        height: 62vh;
        cursor: pointer;
        transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        overflow: hidden;
        position: relative;
    }

    .asset-header-menu {
        display: flex;
        justify-content: space-between;
        gap: 15px;
    }

    .asset-type-card.expanded {
        position: fixed !important;
        top: 5vh !important;
        left: 5vw !important;
        width: 90vw !important;
        height: 90vh !important;
        z-index: 99999 !important;
        opacity: 1 !important;
        visibility: visible !important;
        background: #0f1113;
        transform: none !important;
    }

    .expanded .asset-label { font-size: 1.2rem; color: #00c087; }

    .expanded .asset-value { font-size: 2rem; }

    .expanded .asset-body { margin: 20px; }

    .asset-header {
        padding: 14px 16px;
        display: flex;
        justify-content: space-between;
        align-items: center;
        background: rgba(255, 255, 255, 0.01);
        border-bottom: 1px solid rgba(45, 51, 59, 0.5);
    }

    .asset-label {
        font-size: 0.75rem;
        color: #fff;
        text-transform: uppercase;
        letter-spacing: 0.12em;
        font-weight: 600;
    }

    .action-btn {
        background: transparent;
        border: none;
        color: #444c56;
        cursor: pointer;
        padding: 4px;
        border-radius: 4px;
        transition: 0.2s;
    }

    .delete-btn:hover {
        color: #f87171 !important;
        background: rgba(248, 113, 113, 0.1);
    }

    .expand-btn:hover, .add-item-btn:hover {
        color: #00c087 !important;
        background: rgba(0, 192, 135, 0.1);
    }

    .asset-body {
        flex: 1;
        display: flex;
        background: #050505;
        margin: 10px 12px;
        border: 1px solid #23282e;
        border-radius: 8px;
        overflow: hidden;
        box-shadow: inset 0 0 20px rgba(0, 0, 0, 0.8);
    }

    .asset-scale {
        width: 14px;
        height: 100%;
        border-right: 1px solid rgba(45, 51, 59, 0.4);
        background: repeating-linear-gradient(
            to bottom,
            #444c56 0px, #444c56 1px,
            transparent 1px, transparent 25%
        );
        position: relative;
    }

    .asset-scale::after {
        content: "";
        position: absolute;
        inset: 0;
        background: repeating-linear-gradient(
            to bottom,
            #2d333b 0px, #2d333b 1px,
            transparent 1px, transparent 5%
        );
        opacity: 0.5;
    }

    .asset-container {
        flex: 1;
        position: relative;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .grid-overlay {
        position: absolute;
        inset: 0;
        background-image:
            linear-gradient(rgba(45, 51, 59, 0.1) 1px, transparent 1px),
            linear-gradient(90deg, rgba(45, 51, 59, 0.1) 1px, transparent 1px);
        background-size: 15px 15px;
    }

    .assets-bubbles-stage {
        position: absolute;
        inset: 0;
        z-index: 1;
        overflow: hidden;
    }

    .asset-bubble {
        border-radius: 50%;
        background: rgba(45, 51, 59, 0.20);
        border: 2px solid rgba(45, 51, 59, 0.20);
        box-sizing: border-box;
        position: absolute;
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: hidden;
        transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1), border-color 0.2s;
        flex-shrink: 0;
    }

    .asset-bubble.profit {
        background: rgba(0, 192, 135, 0.20);
        border-color: rgba(0, 192, 135, 0.20);
    }

    .asset-bubble.loss {
        background: rgba(248, 113, 113, 0.20);
        border-color: rgba(248, 113, 113, 0.20);
    }

    .asset-bubble.no-price {
        border-color: rgba(45, 51, 59, 0.20);
    }

    .asset-bubble:hover {
        transform: scale(1.04);
        z-index: 10;
    }

    .bubble-core {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        border-radius: 50%;
        background: #141619;
        border: 2px solid rgba(45, 51, 59, 0.45);
        box-sizing: border-box;
        pointer-events: none;
        z-index: 0;
    }

    .asset-bubble.profit .bubble-core {
        background: rgba(0, 192, 135, 0.45);
        border-color: rgba(0, 192, 135, 0.45);
    }

    .asset-bubble.loss .bubble-core {
        background: rgba(248, 113, 113, 0.45);
        border-color: rgba(248, 113, 113, 0.45);
    }

    .asset-bubble.no-price .bubble-core {
        background: rgba(45, 51, 59, 0.45);
        border-color: rgba(45, 51, 59, 0.45);
    }

    .bubble-ticker {
        font-size: clamp(0.6rem, 2.5vw, 1.3rem);
        font-weight: 700;
        color: #ffffff;
        letter-spacing: 0.02em;
        padding: 0 8px;
        text-align: center;
        max-width: 90%;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        user-select: none;
        text-shadow: 0 2px 4px rgba(0, 0, 0, 0.85), 0 1px 1px rgba(0, 0, 0, 0.9);
    }

    .bubble-value {
        font-size: clamp(0.6rem, 2vw, 1.1rem);
        font-weight: 700;
        color: #ffffff;
        font-family: 'Segoe UI', 'Trebuchet MS', sans-serif;
        max-width: 92%;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        padding: 0 8px;
        letter-spacing: 0.02em;
        text-shadow:
            0 2px 5px rgba(0, 0, 0, 0.85),
            0 1px 1px rgba(0, 0, 0, 0.9),
            0 0 2px rgba(0, 0, 0, 0.6);
    }

    .bubble-percent {
        font-size: clamp(0.55rem, 1.8vw, 1rem);
        font-weight: 700;
        font-family: 'Segoe UI', 'Trebuchet MS', sans-serif;
        white-space: nowrap;
        text-shadow: 0 2px 4px rgba(0, 0, 0, 0.85), 0 1px 1px rgba(0, 0, 0, 0.9);
    }

    .bubble-percent.profit { color: #00c087; }
    .bubble-percent.loss { color: #f87171; }
    .bubble-percent.no-price { color: #64748b; }

    .bubble-text {
        position: relative;
        z-index: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 4px;
        min-width: 0;
        max-width: 94%;
        pointer-events: none;
    }

    .prices-spinner {
        position: absolute;
        top: 10px;
        right: 10px;
        z-index: 5;
        color: #444c56;
        font-size: 0.9rem;
    }

    .content-placeholder {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
        color: #444c56;
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.1em;
    }

    .asset-footer {
        padding: 16px;
        display: flex;
        justify-content: space-between;
        align-items: flex-end;
    }

    .footer-info {
        display: flex;
        flex-direction: row;
        width: 100%;
        justify-content: space-between;
        align-items: center;
    }

    .footer-label {
        font-size: 0.8rem;
        color: #444c56;
        text-transform: uppercase;
        font-weight: 700;
    }

    .asset-value {
        font-size: 1.1rem;
        color: #fff;
        font-weight: 700;
        font-family: 'Courier New', monospace;
    }

    .bubble-context-menu {
        position: fixed;
        background: #141619;
        border: 1px solid #2d333b;
        border-radius: 8px;
        padding: 4px;
        min-width: 140px;
        z-index: 999999 !important;
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.7);
    }

    .menu-item {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 12px;
        color: #94a3b8;
        font-size: 0.75rem;
        cursor: pointer;
        border-radius: 4px;
        transition: 0.15s ease;
    }

    .menu-item:hover {
        background: rgba(255, 255, 255, 0.03);
        color: #fff;
    }

    .menu-item.delete:hover {
        background: rgba(248, 113, 113, 0.1);
        color: #f87171;
    }

    .fade-scale-enter-active, .fade-scale-leave-active {
        transition: opacity 0.15s, transform 0.15s;
    }

    .fade-scale-enter-from, .fade-scale-leave-to {
        opacity: 0;
        transform: scale(0.95);
    }

    .menu-divider {
        height: 1px;
        background: #2d333b;
        margin: 4px 0;
    }

    .asset-creation-layer {
        z-index: 1000000 !important;
    }

    .bubble-tooltip {
        position: fixed;
        background: #141619;
        border: 1px solid #2d333b;
        border-radius: 8px;
        padding: 6px 10px;
        min-width: 130px;
        z-index: 1000001 !important;
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.75);
        font-size: 0.75rem;
        transform: translate(14px, 14px);
        pointer-events: none;
        white-space: nowrap;
        color: #fff;
    }

    .tooltip-ticker {
        font-weight: 700;
        letter-spacing: 0.02em;
    }

    .tooltip-value {
        font-family: 'Segoe UI', 'Trebuchet MS', sans-serif;
        font-weight: 700;
        margin-top: 2px;
    }

    .tooltip-percent {
        font-family: 'Segoe UI', 'Trebuchet MS', sans-serif;
        font-weight: 700;
        margin-top: 2px;
    }

    .tooltip-percent.profit { color: #00c087; }
    .tooltip-percent.loss { color: #f87171; }
    .tooltip-percent.no-price { color: #64748b; }

    .tooltip-cost {
        margin-top: 3px;
        font-size: 0.68rem;
        color: #94a3b8;
        border-top: 1px solid #2d333b;
        padding-top: 3px;
    }
</style>
