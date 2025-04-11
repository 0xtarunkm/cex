"use client"

import React, { useState } from 'react';
import { OrderBookLevel, Trade } from '@/types';

// --- Mock Data ---
const mockBids: OrderBookLevel[] = [
    { price: '112.94', size: '532.29', total: '532.29', isBid: true },
    { price: '112.93', size: '201.15', total: '733.44', isBid: true },
    { price: '112.92', size: '148.50', total: '881.94', isBid: true },
    { price: '112.91', size: '98.32', total: '980.26', isBid: true },
    { price: '112.90', size: '245.67', total: '1225.93', isBid: true },
    { price: '112.89', size: '132.45', total: '1358.38', isBid: true },
    { price: '112.88', size: '175.21', total: '1533.59', isBid: true },
    { price: '112.87', size: '93.76', total: '1627.35', isBid: true },
    { price: '112.86', size: '156.89', total: '1784.24', isBid: true },
    { price: '112.85', size: '218.45', total: '2002.69', isBid: true },
    { price: '112.84', size: '189.32', total: '2192.01', isBid: true },
    { price: '112.83', size: '114.67', total: '2306.68', isBid: true },
];

const mockAsks: OrderBookLevel[] = [
    { price: '112.95', size: '431.56', total: '431.56', isBid: false },
    { price: '112.96', size: '176.23', total: '607.79', isBid: false },
    { price: '112.97', size: '210.45', total: '818.24', isBid: false },
    { price: '112.98', size: '98.76', total: '917.00', isBid: false },
    { price: '112.99', size: '342.87', total: '1259.87', isBid: false },
    { price: '113.00', size: '187.65', total: '1447.52', isBid: false },
    { price: '113.01', size: '124.39', total: '1571.91', isBid: false },
    { price: '113.02', size: '203.74', total: '1775.65', isBid: false },
    { price: '113.03', size: '167.82', total: '1943.47', isBid: false },
];

const mockTrades: Trade[] = [
    { id: '1', price: '112.94', size: '0.42', timestamp: Date.now() - 5000, side: "BUY" },
    { id: '2', price: '112.96', size: '0.15', timestamp: Date.now() - 10000, side: "SELL" },
    { id: '3', price: '112.92', size: '0.35', timestamp: Date.now() - 15000, side: "BUY" },
    { id: '4', price: '112.91', size: '0.27', timestamp: Date.now() - 25000, side: "BUY" },
    { id: '5', price: '112.99', size: '0.51', timestamp: Date.now() - 35000, side: "SELL" },
    { id: '6', price: '113.01', size: '0.19', timestamp: Date.now() - 45000, side: "SELL" },
    { id: '7', price: '112.87', size: '0.33', timestamp: Date.now() - 55000, side: "BUY" },
    { id: '8', price: '112.85', size: '0.44', timestamp: Date.now() - 65000, side: "BUY" },
    { id: '9', price: '112.97', size: '0.28', timestamp: Date.now() - 75000, side: "SELL" },
    { id: '10', price: '113.02', size: '0.37', timestamp: Date.now() - 85000, side: "SELL" },
];
// --- End Mock Data ---

interface OrderBookTradesProps {
    bids?: OrderBookLevel[];
    asks?: OrderBookLevel[];
    trades?: Trade[];
}

const OrderBookTrades: React.FC<OrderBookTradesProps> = ({ 
    bids = mockBids,
    asks = mockAsks,
    trades = mockTrades
}) => {
    const [activeTab, setActiveTab] = useState('orderbook');

    // Get formatted time from timestamp
    const formatTime = (timestamp: number) => {
        const date = new Date(timestamp);
        return `${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}:${date.getSeconds().toString().padStart(2, '0')}`;
    };

    // Calculate the depth percentage for bid/ask background visualization
    const getDepthWidth = (size: string, maxSize: number): string => {
        const percentage = (parseFloat(size) / maxSize) * 100;
        return `${percentage}%`;
    };

    // Find the maximum size for visualization scaling
    const maxAskSize = Math.max(...asks.map(ask => parseFloat(ask.size)));
    const maxBidSize = Math.max(...bids.map(bid => parseFloat(bid.size)));

    return (
        <div className="bg-[#111] flex flex-col border border-gray-800 rounded-sm overflow-hidden h-[calc(100vh-115px)]">
            {/* Tabs */}
            <div className="flex border-b border-gray-800">
                <button 
                    onClick={() => setActiveTab('orderbook')}
                    className={`py-2.5 px-5 text-label font-semibold ${activeTab === 'orderbook' ? 'text-white border-b-2 border-green-500' : 'text-gray-400'}`}
                >
                    Order Book
                </button>
                <button 
                    onClick={() => setActiveTab('trades')}
                    className={`py-2.5 px-5 text-label font-semibold ${activeTab === 'trades' ? 'text-white border-b-2 border-green-500' : 'text-gray-400'}`}
                >
                    Trades
                </button>
            </div>

            {/* Order Book View */}
            {activeTab === 'orderbook' && (
                <div className="flex flex-col flex-grow">
                    {/* Column Headers */}
                    <div className="grid grid-cols-3 text-xs text-gray-400 py-2 px-3 font-medium">
                        <div className="text-left">Price</div>
                        <div className="text-right">Size</div>
                        <div className="text-right">Total</div>
                    </div>
                    
                    {/* Asks (Sell Orders) */}
                    <div className="overflow-y-auto scrollbar-thin scrollbar-thumb-gray-700" style={{ maxHeight: '205px' }}>
                        {asks.slice().reverse().map((ask, index) => (
                            <div 
                                key={index} 
                                className="ask-row grid grid-cols-3 text-xs py-1 px-3 hover:bg-gray-800 relative"
                            >
                                <div className="ask-row-bg absolute inset-0" style={{ width: getDepthWidth(ask.size, maxAskSize), backgroundColor: 'var(--color-ask-bg)', zIndex: 0 }}></div>
                                <div className="ask-text text-left relative z-10">{ask.price}</div>
                                <div className="text-right font-medium relative z-10">{ask.size}</div>
                                <div className="text-right font-medium relative z-10">{ask.total}</div>
                            </div>
                        ))}
                    </div>
                    
                    {/* Price Spread */}
                    <div className="grid grid-cols-3 text-xs py-2 px-3 border-y border-gray-800 bg-gray-800/20">
                        <div className="text-left text-gray-300 font-semibold">
                            {(parseFloat(asks[0].price) - parseFloat(bids[0].price)).toFixed(2)} Spread
                        </div>
                        <div className="text-right"></div>
                        <div className="text-right text-gray-400 font-medium">
                            {((parseFloat(asks[0].price) - parseFloat(bids[0].price)) / parseFloat(bids[0].price) * 100).toFixed(3)}%
                        </div>
                    </div>
                    
                    {/* Bids (Buy Orders) */}
                    <div className="overflow-y-auto scrollbar-thin scrollbar-thumb-gray-700" style={{ maxHeight: '215px' }}>
                        {bids.map((bid, index) => (
                            <div 
                                key={index} 
                                className="bid-row grid grid-cols-3 text-xs py-1 px-3 hover:bg-gray-800 relative"
                            >
                                <div className="bid-row-bg absolute inset-0" style={{ width: getDepthWidth(bid.size, maxBidSize), backgroundColor: 'var(--color-bid-bg)', zIndex: 0 }}></div>
                                <div className="bid-text text-left relative z-10">{bid.price}</div>
                                <div className="text-right font-medium relative z-10">{bid.size}</div>
                                <div className="text-right font-medium relative z-10">{bid.total}</div>
                            </div>
                        ))}
                    </div>
                    
                    {/* Footer */}
                    <div className="mt-auto border-t border-gray-800 p-3 text-xs text-gray-400 flex justify-between items-center">
                        <div className="font-medium">Bids: 50.0%</div>
                        <div className="flex gap-4">
                            <button className="hover:text-white">0.01</button>
                            <button className="hover:text-white">0.1</button>
                            <button className="text-white font-semibold">1</button>
                        </div>
                        <div className="font-medium">Asks: 50.0%</div>
                    </div>
                </div>
            )}
            
            {/* Trades View */}
            {activeTab === 'trades' && (
                <div className="flex flex-col flex-grow">
                    {/* Column Headers */}
                    <div className="grid grid-cols-3 text-xs text-gray-400 py-2 px-3 font-medium">
                        <div className="text-left">Price</div>
                        <div className="text-right">Size</div>
                        <div className="text-right">Time</div>
                    </div>
                    
                    {/* Trades List */}
                    <div className="overflow-y-auto scrollbar-thin scrollbar-thumb-gray-700" style={{ maxHeight: '434px' }}>
                        {trades.map((trade, index) => (
                            <div 
                                key={index} 
                                className="grid grid-cols-3 text-xs py-1 px-3 hover:bg-gray-800"
                            >
                                <div className={`text-left ${trade.side === 'BUY' ? 'bid-text' : 'ask-text'}`}>
                                    {trade.price}
                                </div>
                                <div className="text-right font-medium">{trade.size}</div>
                                <div className="text-right text-gray-400">{formatTime(trade.timestamp)}</div>
                            </div>
                        ))}
                    </div>
                    
                    {/* Footer */}
                    <div className="mt-auto border-t border-gray-800 p-3 text-xs text-gray-400 flex justify-between items-center">
                        <div>Last 24h</div>
                        <div className="flex gap-3">
                            <span className="bid-text">Buy: 52.3%</span>
                            <span className="ask-text">Sell: 47.7%</span>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};

export default OrderBookTrades; 