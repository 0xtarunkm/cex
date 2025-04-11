"use client"

import React, { useState } from 'react';
import { Balance } from '@/types'; // Import Balance type
import { EyeIcon, ArrowDownIcon, EllipsisHorizontalIcon } from '@heroicons/react/24/outline';

// --- Mock Data ---
const mockBalances: Balance[] = [
    {
        asset: 'USDC', ticker: 'USD Coin', icon: '💲', // Placeholder icon
        balance: '0', balanceValue: '$0.00',
        available: '0', availableValue: '$0.00',
        lendBorrow: '0', openOrders: '0'
    },
    {
        asset: 'SOL', ticker: 'Solana', icon: ' S ', // Placeholder icon
        balance: '0', balanceValue: '$0.00',
        available: '0', availableValue: '$0.00',
        lendBorrow: '0', openOrders: '0'
    },
    {
        asset: 'BTC', ticker: 'Bitcoin', icon: ' B ', // Placeholder icon
        balance: '0', balanceValue: '$0.00',
        available: '0', availableValue: '$0.00',
        lendBorrow: '0', openOrders: '0'
    },
    {
        asset: 'USDT', ticker: 'USDT', icon: ' T ', // Placeholder icon
        balance: '0', balanceValue: '$0.00',
        available: '0', availableValue: '$0.00',
        lendBorrow: '0', openOrders: '0'
    },
     {
        asset: 'ETH', ticker: 'Ethereum', icon: ' E ', // Placeholder icon
        balance: '0', balanceValue: '$0.00',
        available: '0', availableValue: '$0.00',
        lendBorrow: '0', openOrders: '0'
    },
    {
        asset: 'AAVE', ticker: 'Aave', icon: ' A ',
        balance: '0', balanceValue: '$0.00',
        available: '0', availableValue: '$0.00',
        lendBorrow: null, openOrders: '0'
    },
     {
        asset: 'ACT', ticker: 'Act I The AI Prophecy', icon: 'Ac',
        balance: '0', balanceValue: '$0.00',
        available: '0', availableValue: '$0.00',
        lendBorrow: null, openOrders: '0'
    },
    // Add more mock assets as needed
];
// --- End Mock Data ---

type TabName = 'Balances' | 'Positions' | 'Borrows' | 'Open Orders' | 'Fill History' | 'Order History' | 'Position History';

const BottomTabs: React.FC = () => {
  const [activeTab, setActiveTab] = useState<TabName>('Balances');
  const [hideZeroBalances, setHideZeroBalances] = useState(false);

  const tabs: TabName[] = [
      'Balances', 'Positions', 'Borrows', 'Open Orders', 'Fill History', 'Order History', 'Position History'
  ];

  const filteredBalances = hideZeroBalances
    ? mockBalances.filter(b => parseFloat(b.balance) > 0 || parseFloat(b.available) > 0)
    : mockBalances;

  const renderContent = () => {
    switch (activeTab) {
      case 'Balances':
        return (
          <div className="flex-grow overflow-y-auto scrollbar-thin scrollbar-thumb-gray-600 scrollbar-track-transparent">
             {/* Balances Header */}
            <div className="flex justify-between items-center p-5">
                 <div>
                    <span className="text-gray-400 text-sm block">Your Balances</span>
                    <div className="flex items-center gap-2 mt-1">
                        <span className="text-white text-xl font-semibold">$0.00</span>
                        <span className="text-gray-500 text-xs bg-[#222] px-2 py-1 rounded">US$0.00 0.0%</span>
                        <button className="text-gray-400 hover:text-white"><EyeIcon className="w-5 h-5" /></button>
                    </div>
                </div>
                <label className="flex items-center gap-2 text-sm text-gray-400 cursor-pointer">
                    <input
                        type="checkbox"
                        checked={hideZeroBalances}
                        onChange={() => setHideZeroBalances(!hideZeroBalances)}
                         className="form-checkbox h-4 w-4 text-blue-600 bg-[#222] border-gray-700 rounded focus:ring-blue-500 focus:ring-offset-gray-900"
                    />
                    Hide zero balances
                </label>
            </div>

            {/* Balances Table */}
            <table className="w-full text-sm">
                <thead>
                    <tr className="border-b border-gray-800 text-gray-400">
                        <th className="text-left font-medium px-5 py-3">Asset</th>
                        <th className="text-right font-medium px-5 py-3 flex items-center justify-end gap-1">
                            <ArrowDownIcon className="w-4 h-4" /> Total Balance
                         </th>
                        <th className="text-right font-medium px-5 py-3">Available Balance</th>
                        <th className="text-right font-medium px-5 py-3">Lend & Borrow</th>
                        <th className="text-right font-medium px-5 py-3">Open Orders</th>
                        <th className="text-right font-medium px-5 py-3"></th> {/* Actions column */}
                    </tr>
                </thead>
                <tbody>
                    {filteredBalances.map((bal, index) => (
                        <tr key={bal.asset} className="border-b border-gray-800 hover:bg-[#222]/50">
                            <td className="px-5 py-4 flex items-center gap-3">
                                <span className="inline-flex items-center justify-center w-8 h-8 bg-[#222] rounded-full text-sm font-medium text-gray-300">
                                    {bal.icon || bal.asset.charAt(0)}
                                </span>
                                <div>
                                    <div className="text-white font-medium">{bal.ticker}</div>
                                    <div className="text-gray-400">{bal.asset}</div>
                                </div>
                            </td>
                            <td className="px-5 py-4 text-right">
                                <div className="text-white font-medium">{bal.balance}</div>
                                <div className="text-gray-400">{bal.balanceValue}</div>
                            </td>
                             <td className="px-5 py-4 text-right">
                                <div className="text-white font-medium">{bal.available}</div>
                                <div className="text-gray-400">{bal.availableValue}</div>
                            </td>
                            <td className="px-5 py-4 text-right text-white font-medium">
                                {bal.lendBorrow ?? '-'}
                             </td>
                             <td className="px-5 py-4 text-right text-white font-medium">
                                 {bal.openOrders}
                             </td>
                             <td className="px-5 py-4 text-right">
                                 <button className="text-blue-500 hover:text-blue-400 font-medium mr-4">Deposit</button>
                                 <button className="text-blue-500 hover:text-blue-400 font-medium mr-4">Withdraw</button>
                                 <button className="text-gray-400 hover:text-white"><EllipsisHorizontalIcon className="w-5 h-5" /></button>
                             </td>
                        </tr>
                    ))}
                </tbody>
            </table>
          </div>
        );
      default:
        return (
            <div className="flex-grow p-5">
                <div className="text-gray-500 text-center py-10">{activeTab} Content Placeholder</div>
            </div>
        );
    }
  };

  return (
    <footer className="h-48 border-t border-gray-800 flex flex-col text-sm bg-[#111]">
      {/* Tabs */}
      <div className="flex border-b border-gray-800">
         {tabs.map(tab => (
            <button
                key={tab}
                onClick={() => setActiveTab(tab)}
                className={`px-5 py-3 ${activeTab === tab ? 'text-white font-semibold border-b-2 border-white -mb-px' : 'text-gray-400 hover:text-white'}`}
            >
                {tab}
            </button>
         ))}
      </div>

      {/* Tab Content */}
      {renderContent()}
    </footer>
  );
};

export default BottomTabs;
