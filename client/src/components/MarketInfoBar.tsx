"use client"

import React from 'react';
import { ChevronDownIcon, ChevronRightIcon } from '@heroicons/react/24/solid';

const MarketInfoBar = () => {
  return (
    <div className="h-14 border-b border-gray-800 flex items-center px-6 text-sm bg-[#111]">
      <div className="flex items-center gap-2 mr-8">
        <div className="w-7 h-7 bg-blue-500 rounded-full flex items-center justify-center text-xs font-bold">S</div>
        <span className="font-semibold text-lg">SOL/USDC</span>
      </div>

      <div className="flex items-center gap-7 text-sm">
        <div className="flex flex-col">
          <span className="font-bold text-xl text-green-500">112.63</span>
          <span className="text-gray-400">$112.63</span>
        </div>
      </div>
      <button className="ml-auto p-1.5 text-gray-400 hover:text-white">
        <ChevronRightIcon className="w-5 h-5" />
      </button>
    </div>
  );
};

export default MarketInfoBar; 