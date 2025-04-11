import Header from "@/components/Header";
import MarketInfoBar from "@/components/MarketInfoBar";
import OrderBookTrades from "@/components/OrderBookTrades";
import OrderEntry from "@/components/OrderEntry";
import { Chart } from "@/components/Chart";
import BottomTabs from "@/components/BottomTabs";

export default function Home() {
  return (
    <div className="grid grid-rows-[max-content_max-content_1fr] grid-cols-[3fr_1fr] min-h-screen bg-black text-gray-100 font-sans gap-2">
      <Header />
      <MarketInfoBar />

      <div className="grid grid-cols-[3fr_1fr] min-h-[600px]">
        <Chart />
        <OrderBookTrades />
      </div>

      <div className="row-span-2 self-start">
        <OrderEntry />
      </div>
      <BottomTabs />
    </div>
  );
}
