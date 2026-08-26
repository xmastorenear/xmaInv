export interface Strategy {
    id: number;
    name: string;
    color: string;
    distribution: Record<number, number>;
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
    strategy_id: number; // Link to a strategy
    name: string;        // Custom name (Stocks, Crypto, etc.)
    total_value: number; // Total value in this group
}

export interface Asset {
    id: number;
    group_id: number;
    ticker: string;
    amount: number;
    buy_price: number;
    uid: string | null;
}

export interface Transaction {
    id: number;
    source_id: number;
    amount: number;
    timestamp: string;
    description: string;
}

// Response interface from the get_data command
export interface AppDataResponse {
    strategy: Strategy | null;
    all_strategies: Strategy[];
    sources: Source[];
    asset_groups: AssetGroup[];
    transactions: Transaction[];
    assets: Asset[];
    tbank_token: string | null;
}

// Instrument from the T-Bank API search (search_instruments command)
export interface InstrumentCard {
    ticker: string;
    name: string;
    class_code: string;
    instrument_type: string;
    currency: string;
    uid: string;
    price: number | null;
}

export interface AssetPrice {
    asset_id: number;
    price: number | null;
    stale: boolean;
}
