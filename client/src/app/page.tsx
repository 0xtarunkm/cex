import Image from "next/image";
import Header from "@/components/Header";
import MarketInfoBar from "@/components/MarketInfoBar";
import Chart from "@/components/Chart";
import OrderBookTrades from "@/components/OrderBookTrades";
import OrderEntry from "@/components/OrderEntry";
import BottomTabs from "@/components/BottomTabs";

export default function Home() {
  return (
    <div className="flex flex-col min-h-screen bg-black text-gray-100 font-sans">
      <Header />
      <MarketInfoBar />

      {/* Main Content Area - Using auto height and better alignment */}
      <div className="bg-black grid grid-cols-12 gap-0.5">
        {/* Chart takes 7 columns */}
        <div className="col-span-7">
          <Chart />
        </div>
        
        {/* Order Book takes 2.5 columns */}
        <div className="col-span-2">
          <OrderBookTrades />
        </div>
        
        {/* Order Entry takes 3 columns */}
        <div className="col-span-3">
          <OrderEntry />
        </div>
      </div>

      {/* BottomTabs can now extend below the viewport */}
      <BottomTabs />

      {/* Bottom Tabs Placeholder */}
      <footer className="h-48 border-t border-gray-700">Bottom Tabs / Balances</footer>
    </div>
  );
}
