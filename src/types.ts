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
    transactions: Transaction[];
}
