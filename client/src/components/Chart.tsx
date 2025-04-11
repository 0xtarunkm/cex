"use client"

import React from 'react';
import {
    ClockIcon,
    ArrowsRightLeftIcon,
    Cog6ToothIcon,
    EyeIcon,
    CameraIcon,
    ArrowUturnLeftIcon,
    ArrowUturnRightIcon,
    CalendarDaysIcon
} from '@heroicons/react/24/outline';
import { Kline } from '@/types';

// --- Mock Data ---
const mockKlines: Kline[] = [
    { timestamp: Date.now() - 3600000 * 11, open: '110.50', high: '111.80', low: '110.20', close: '111.50', volume: '1500' },
    { timestamp: Date.now() - 3600000 * 10, open: '111.50', high: '112.90', low: '111.00', close: '112.78', volume: '2500' },
    { timestamp: Date.now() - 3600000 * 9, open: '112.78', high: '113.31', low: '112.08', close: '112.95', volume: '1800' },
    { timestamp: Date.now() - 3600000 * 8, open: '112.95', high: '114.50', low: '112.80', close: '114.20', volume: '3200' },
    { timestamp: Date.now() - 3600000 * 7, open: '114.20', high: '114.80', low: '112.50', close: '112.78', volume: '2800' },
    { timestamp: Date.now() - 3600000 * 6, open: '112.78', high: '112.90', low: '111.20', close: '111.50', volume: '1900' },
    { timestamp: Date.now() - 3600000 * 5, open: '111.50', high: '112.50', low: '111.30', close: '112.30', volume: '2100' },
    { timestamp: Date.now() - 3600000 * 4, open: '112.30', high: '113.00', low: '112.20', close: '112.70', volume: '2300' },
    { timestamp: Date.now() - 3600000 * 3, open: '112.70', high: '114.00', low: '112.60', close: '113.80', volume: '2700' },
    { timestamp: Date.now() - 3600000 * 2, open: '113.80', high: '114.20', low: '112.97', close: '113.00', volume: '2400' },
    { timestamp: Date.now() - 3600000 * 1, open: '113.00', high: '113.20', low: '111.90', close: '112.94', volume: '1800' },
];
// --- End Mock Data ---

interface ChartProps {
    klines?: Kline[];
}

const Chart: React.FC<ChartProps> = ({ klines = mockKlines }) => {
  return (
    <div className="bg-[#111] flex flex-col border border-gray-800 rounded-sm overflow-hidden h-[calc(100vh-115px)]">
      {/* Chart Controls */}
      <div className="flex items-center justify-between p-3 border-b border-gray-800 text-sm">
        <div className="flex gap-6">
          <button className="text-white font-semibold">Chart</button>
          <button className="text-gray-400 hover:text-white">Depth</button>
          <button className="text-gray-400 hover:text-white">Margin</button>
        </div>
        <div>{/* Potentially other controls here */}</div>
      </div>
      <div className="flex items-center justify-between px-3 py-2 border-b border-gray-800 text-sm text-gray-300">
         <div className="flex items-center gap-5">
          <button className="text-white font-medium">1h</button>
          <button className="hover:text-white"><ClockIcon className="w-5 h-5" /></button>
          <button className="hover:text-white font-medium">fx Indicators</button>
          <button className="hover:text-white">OL</button>
          <button className="hover:text-white">TE</button>
          <button className="hover:text-white"><ArrowUturnLeftIcon className="w-5 h-5" /></button>
          <button className="hover:text-white"><ArrowUturnRightIcon className="w-5 h-5" /></button>
        </div>
        <div className="flex items-center gap-5">
           <button className="hover:text-white"><ArrowsRightLeftIcon className="w-5 h-5" /></button>
           <button className="hover:text-white"><EyeIcon className="w-5 h-5" /></button>
           <button className="hover:text-white"><CameraIcon className="w-5 h-5" /></button>
           <button className="hover:text-white"><Cog6ToothIcon className="w-5 h-5" /></button>
          <button className="ml-2 hover:text-white font-medium">Reset</button>
        </div>
      </div>

      {/* Chart Area - with Candlesticks */}
      <div className="flex-grow relative bg-[#111] p-0" style={{ minHeight: "470px" }}>
        {/* Y-axis price labels */}
        <div className="absolute top-0 right-0 bottom-24 flex flex-col justify-between text-xs text-gray-400 px-2 py-1 w-12 text-right">
          <div>124.00</div>
          <div>120.00</div>
          <div>116.00</div>
          <div>112.94</div>
          <div>108.00</div>
          <div>104.00</div>
          <div>100.00</div>
          <div>96.00</div>
          <div>80.00</div>
          <div>40.00</div>
        </div>
        
        {/* Background grid lines */}
        <div className="absolute inset-0 grid grid-rows-8 grid-cols-12 pointer-events-none">
          {Array(8).fill(0).map((_, i) => (
            <div key={`hline-${i}`} className="border-b border-gray-800 col-span-12"></div>
          ))}
          {Array(12).fill(0).map((_, i) => (
            <div key={`vline-${i}`} className="border-r border-gray-800 row-span-8"></div>
          ))}
        </div>
        
        {/* Candlestick chart */}
        <div className="absolute top-4 left-4 right-12 bottom-24 flex items-end">
          <div className="flex items-end justify-between h-full w-full">
            {klines.map((kline, i) => {
              const isGreen = parseFloat(kline.close) >= parseFloat(kline.open);
              const height = 200; // Simulate chart height
              const bodyHeight = Math.abs(parseFloat(kline.close) - parseFloat(kline.open)) / 0.1 * 5;
              const wickHeight = Math.abs(parseFloat(kline.high) - parseFloat(kline.low)) / 0.1 * 5;
              const positionFromBottom = (parseFloat(kline.low) - 110) / 0.1 * 5;
              
              return (
                <div key={i} className="flex flex-col items-center justify-end relative" style={{ height: `${height}px`, width: '20px' }}>
                  {/* Candle wick */}
                  <div 
                    className={`w-px ${isGreen ? 'bg-green-500' : 'bg-red-500'}`} 
                    style={{ 
                      height: `${wickHeight}px`,
                      position: 'absolute',
                      bottom: `${positionFromBottom}px` 
                    }}
                  ></div>
                  
                  {/* Candle body */}
                  <div 
                    className={`w-3 ${isGreen ? 'bg-green-500' : 'bg-red-500'}`} 
                    style={{ 
                      height: `${Math.max(bodyHeight, 1)}px`,
                      position: 'absolute',
                      bottom: `${positionFromBottom + (isGreen ? 0 : bodyHeight)}px`
                    }}
                  ></div>
                </div>
              );
            })}
          </div>
        </div>
        
        {/* Current price marker */}
        <div className="absolute right-12 text-green-500 text-xs font-semibold" style={{ top: '35%' }}>
          112.94
        </div>
        <div className="absolute right-12 left-0 border-t border-dashed border-green-500/30" style={{ top: '35%' }}></div>
        
        {/* Candlestick info */}
        <div className="absolute top-3 left-3 text-sm text-gray-400">
          SOL_USDC · 1h · Backpack
        </div>
      
        {/* Price metrics */}
        <div className="absolute top-10 left-3 flex gap-4 text-gray-300 text-sm">
          <span>O<span className="text-green-500 ml-1">112.64</span></span>
          <span>H<span className="text-green-500 ml-1">112.97</span></span>
          <span>L<span className="text-green-500 ml-1">112.47</span></span>
          <span>C<span className="text-green-500 ml-1">112.94</span></span>
          <span className="ml-2 text-green-500">0.34 (+0.30%)</span>
        </div>
        
        {/* Volume area */}
        <div className="absolute bottom-0 left-0 right-12 h-24 border-t border-gray-800">
          <div className="text-xs text-gray-400 pl-3 pt-1">Volume SMA <span className="text-green-500">346.32</span></div>
          <div className="flex justify-between h-20 items-end px-4">
            {Array(12).fill(0).map((_, i) => (
              <div key={`vol-${i}`} className={`w-3 ${i % 3 === 0 ? 'bg-green-500/50' : 'bg-red-500/50'}`} style={{ height: `${10 + Math.random() * 70}%` }}></div>
            ))}
          </div>
        </div>
        
        {/* X-axis time labels */}
        <div className="absolute bottom-0 left-3 right-12 flex justify-between text-xs text-gray-400">
          <div>6</div>
          <div>7</div>
          <div>8</div>
          <div>9</div>
          <div>10</div>
          <div>11</div>
        </div>
      </div>

      {/* Chart Footer Controls */}
      <div className="flex items-center justify-between px-3 py-2 border-t border-gray-800 text-sm text-gray-300">
        <div className="flex gap-3">
          <button className="hover:text-white">All</button>
          <button className="hover:text-white">1y</button>
          <button className="hover:text-white">6m</button>
          <button className="hover:text-white">3m</button>
          <button className="hover:text-white">1m</button>
          <button className="hover:text-white">5d</button>
          <button className="text-white font-medium">1d</button>
          <button className="ml-2 hover:text-white"><CalendarDaysIcon className="w-5 h-5" /></button>
        </div>
        <div className="flex gap-5 items-center">
             <span className="font-medium">23:09:59 (UTC)</span>
             <button className="hover:text-white">%</button>
             <button className="hover:text-white">log</button>
             <button className="text-white">auto</button>
        </div>
      </div>
    </div>
  );
};

export default Chart; 