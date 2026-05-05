export interface Strategy {
    id: number;
    name: string;
    color: string;
}

export interface Source {
    id: number;
    strategy_id: number;
    name: string;
    icon_url: string;
    total_balance: number;
    profit_loss: number;
}

export interface AssetGroup {
    id: number;
    strategy_id: number; // Привязка к стратегии
    name: string;        // Пользовательское название (Акции, Крипта и т.д.)
    total_value: number; // Общая сумма в этой группе
}

export interface Asset {
    id: number;
    group_id: number;
    ticker: string;
    amount: number;
    buy_price: number;
}

export interface Transaction {
    id: number;
    source_id: number;
    amount: number;
    timestamp: string;
    description: string;
}

// Интерфейс ответа от команды get_data
export interface AppDataResponse {
    strategy: Strategy | null;
    all_strategies: Strategy[];
    sources: Source[];
    asset_groups: AssetGroup[];
    transactions: Transaction[];
    assets: Asset[];
}
