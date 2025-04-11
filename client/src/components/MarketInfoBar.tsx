"use client"

import React from 'react';
import { ChevronDownIcon, ChevronRightIcon } from '@heroicons/react/24/solid';

const MarketInfoBar: React.FC = () => {
  return (
    <div className="h-14 border-b border-gray-800 flex items-center px-6 text-sm bg-[#111]">
      <div className="flex items-center gap-2 mr-8">
        <div className="w-7 h-7 bg-blue-500 rounded-full flex items-center justify-center text-xs font-bold">S</div>
        <span className="font-semibold text-lg">SOL/USDC</span>
        <ChevronDownIcon className="w-5 h-5 text-gray-400" />
      </div>

      <div className="flex items-center gap-7 text-sm">
        <div className="flex flex-col">
          <span className="font-bold text-xl text-green-500">112.63</span>
          <span className="text-gray-400">$112.63</span>
        </div>
        <div className="flex flex-col">
          <span className="text-gray-400">24H Change</span>
          <div className="text-red-500 font-medium">-6.53 -5.48%</div>
        </div>
         <div className="flex flex-col">
          <span className="text-gray-400">24H High</span>
          <div className="text-white font-medium">119.41</div>
        </div>
         <div className="flex flex-col">
          <span className="text-gray-400">24H Low</span>
          <div className="text-white font-medium">106.00</div>
        </div>
        <div className="flex flex-col">
          <span className="text-gray-400">24H Volume (USDC)</span>
          <div className="text-white font-medium">18,354,295.46</div>
        </div>
        <div className="border-l border-gray-800 pl-6 ml-2 flex flex-col">
          <span className="text-gray-400">Lend APY (SOL / USDC)</span>
          <div className="font-medium"><span className="text-green-500">0.47%</span> / <span className="text-green-500">1.58%</span></div>
        </div>
        <div className="flex flex-col">
          <span className="text-gray-400">Borrow APY (SOL / USDC)</span>
           <div className="font-medium"><span className="text-red-500">2.74%</span> / <span className="text-red-500">5.07%</span></div>
        </div>
      </div>
      <button className="ml-auto p-1.5 text-gray-400 hover:text-white">
        <ChevronRightIcon className="w-5 h-5" />
      </button>
    </div>
  );
};

export default MarketInfoBar; 