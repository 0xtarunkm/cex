"use client"

import React, { useState } from 'react';
import { CheckIcon, InformationCircleIcon, XMarkIcon } from '@heroicons/react/24/outline';
import { Bars3Icon, BanknotesIcon } from '@heroicons/react/24/solid'; // Using solid for filled icons

const OrderEntry: React.FC = () => {
  const [marginChecked, setMarginChecked] = useState(false);

  return (
    <div className="bg-[#111] flex flex-col p-5 border border-gray-800 rounded-sm h-full overflow-auto">
      {/* Buy/Sell Tabs */}
      <div className="flex mb-6">
        <button className="flex-1 py-3 bg-green-600 text-white font-semibold rounded-l-md">Buy</button>
        <button className="flex-1 py-3 bg-[#222] text-gray-300 hover:bg-gray-700 rounded-r-md font-semibold">Sell</button>
      </div>

      {/* Order Type Tabs */}
      <div className="flex items-center gap-6 mb-6 text-label">
        <button className="text-white font-semibold">Limit</button>
        <button className="hover:text-white text-gray-400 font-medium">Market</button>
        <button className="hover:text-white text-gray-400 font-medium">Conditional</button>
      </div>

      <div className="mb-5 text-right text-sm text-gray-400">
          Balance <span className="text-white font-semibold">0 USDC</span>
      </div>

      {/* Price Input */}
      <div className="mb-5">
        <div className="flex justify-between items-center mb-2">
          <label htmlFor="price" className="text-gray-400 font-medium">Price</label>
          <button className="text-blue-500 font-semibold">Mid</button>
        </div>
        <div className="relative">
            <input
              type="text"
              id="price"
              value="112.96"
              className="w-full bg-black border border-gray-800 rounded-md p-3 pr-10 text-right text-xl font-semibold"
            />
            <div className="absolute right-3 top-1/2 transform -translate-y-1/2">
              <div className="text-blue-500">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                  <path d="M12 22C17.5228 22 22 17.5228 22 12C22 6.47715 17.5228 2 12 2C6.47715 2 2 6.47715 2 12C2 17.5228 6.47715 22 12 22Z" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                  <path d="M12 17V17.01" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                  <path d="M9.5 9.5C9.5 8.09997 10.6 7 12 7C13.4 7 14.5 8.09997 14.5 9.5C14.5 10.9 13.4 12 12 12C12 12 12 13 12 14" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                </svg>
              </div>
            </div>
        </div>
      </div>

      {/* Quantity Input */}
      <div className="mb-5">
        <label htmlFor="quantity" className="block text-gray-400 font-medium mb-2">Quantity</label>
         <div className="relative">
            <input
              type="text"
              id="quantity"
              value="100"
              className="w-full bg-black border border-gray-800 rounded-md p-3 pr-10 text-right text-xl font-semibold"
            />
            <div className="absolute right-3 top-1/2 transform -translate-y-1/2">
              <div className="text-blue-500">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                  <path d="M4 6H20M4 12H20M4 18H20" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                </svg>
              </div>
            </div>
         </div>
      </div>

      {/* Quantity Slider */}
      <div className="mb-8">
         <div className="relative w-full mb-1">
           <input 
             type="range" 
             min="0" 
             max="100" 
             defaultValue="0" 
             className="w-full h-1 bg-gray-800 rounded-lg appearance-none cursor-pointer accent-blue-500" 
           />
         </div>
         <div className="flex justify-between text-xs text-gray-400 mt-2">
            <span>0</span>
            <span>25%</span>
            <span>50%</span>
            <span>75%</span>
            <span>100%</span>
        </div>
      </div>

      {/* Order Value */}
      <div className="mb-5">
        <div className="flex justify-between items-center mb-1">
            <span className="text-gray-400 font-medium">Order Value</span>
            <div className="text-blue-500">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M12 22C17.5228 22 22 17.5228 22 12C22 6.47715 17.5228 2 12 2C6.47715 2 2 6.47715 2 12C2 17.5228 6.47715 22 12 22Z" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                <path d="M12 17V17.01" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                <path d="M9.5 9.5C9.5 8.09997 10.6 7 12 7C13.4 7 14.5 8.09997 14.5 9.5C14.5 10.9 13.4 12 12 12C12 12 12 13 12 14" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            </div>
        </div>
        <div className="text-right text-xl font-semibold">11,296</div>
      </div>

      <div className="text-sm text-gray-400 mb-1">
        Borrow Amount (<span className="text-red-500 font-semibold">5.06%</span>) 
        <span className="text-white ml-2 font-semibold">11,296 USDC</span>
      </div>
      <div className="text-sm text-gray-400 mb-8">
        Max Buying Amount <span className="text-white ml-2 font-semibold">0.00 SOL</span>
      </div>

      {/* Buy Button */}
      <button className="w-full bg-green-600 hover:bg-green-700 text-white font-semibold py-4 rounded-md mb-6 text-base">
        Buy
      </button>

      {/* Order Options */}
      <div className="flex justify-between items-center text-sm text-gray-400 mb-5">
        <div className="flex gap-5">
            <label className="flex items-center gap-1.5 cursor-pointer">
                <input type="checkbox" className="form-checkbox h-4 w-4 rounded bg-gray-800 border-gray-700 text-blue-600 focus:ring-blue-500 focus:ring-offset-gray-900" /> Post Only
            </label>
            <label className="flex items-center gap-1.5 cursor-pointer">
                <input type="checkbox" className="form-checkbox h-4 w-4 rounded bg-gray-800 border-gray-700 text-blue-600 focus:ring-blue-500 focus:ring-offset-gray-900" /> IOC
            </label>
             <label className="flex items-center gap-1.5 cursor-pointer">
                <input 
                  type="checkbox" 
                  checked={marginChecked}
                  onChange={() => setMarginChecked(!marginChecked)}
                  className="form-checkbox h-4 w-4 rounded bg-gray-800 border-gray-700 text-blue-600 focus:ring-blue-500 focus:ring-offset-gray-900" 
                /> Margin
            </label>
        </div>
      </div>

      {/* Cross Margin Overview - Only show when margin is checked */}
      {marginChecked && (
        <div className="border-t border-gray-800 pt-6 mt-auto">
          <div className="flex justify-between items-center mb-4">
              <div className="flex items-center gap-1 text-sm text-gray-400">
                  Cross Margin Overview <InformationCircleIcon className="w-4 h-4" />
              </div>
              <div className="flex items-center gap-3 text-sm">
                  <span className="text-white bg-[#222] px-2 py-1 rounded">10x</span>
                  {/* Lightning icon */}
                  <button className="text-green-500 font-bold text-lg">⚡</button>
              </div>
          </div>
          
          <div className="flex justify-between text-sm mb-2">
              <span className="text-gray-400">Initial Margin</span>
              <span className="text-white font-medium">0%</span>
          </div>
          <div className="flex justify-between text-sm mb-2">
              <span className="text-gray-400">Maintenance Margin</span>
              <span className="text-white font-medium">0%</span>
          </div>
          
          <div className="h-1 bg-gray-800 rounded my-4"></div>
          
          <div className="flex justify-between text-sm mb-2">
              <span className="text-gray-400">Equity Total</span>
              <span className="text-white font-medium">$0.00</span>
          </div>
          <div className="flex justify-between text-sm mb-2">
              <span className="text-gray-400">Equity Available</span>
              <span className="text-white font-medium">$0.00</span>
          </div>
          <div className="flex justify-between text-sm mb-2">
              <span className="text-gray-400">Open PnL</span>
              <span className="text-white font-medium">$0.00</span>
          </div>
        </div>
      )}
    </div>
  );
};

export default OrderEntry; 