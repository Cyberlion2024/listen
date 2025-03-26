import { useTranslation } from "react-i18next";

interface Faster100xDisplayProps {
  data: {
    status: string;
    message?: string;
    holders?: Array<{
      address: string;
      amount: number;
      amount_percentage: number;
    }>;
  };
}

export default function Faster100xDisplay({ data }: Faster100xDisplayProps) {
  const { t } = useTranslation();

  if (data.status === "error") {
    return (
      <div className="bg-red-50 p-4 rounded-lg border border-red-200">
        <p className="text-red-700">{data.message}</p>
      </div>
    );
  }

  if (!data.holders || data.holders.length === 0) {
    return (
      <div className="bg-yellow-50 p-4 rounded-lg border border-yellow-200">
        <p className="text-yellow-700">{t("No holder data available")}</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="bg-white p-6 rounded-lg shadow-sm">
        <h3 className="text-xl font-bold mb-4 text-black border-b pb-2">
          {t("Wallet Concentration Analysis")}
        </h3>
        <div className="space-y-4">
          {data.holders.map((holder) => (
            <div
              key={holder.address}
              className="p-4 rounded-lg border flex justify-between items-center"
            >
              <div className="flex flex-col">
                <span className="font-medium text-black">
                  {holder.address.slice(0, 8)}...{holder.address.slice(-8)}
                </span>
                <span className="text-sm text-gray-600">
                  {holder.amount.toLocaleString()} tokens
                </span>
              </div>
              <div className="text-right">
                <span className="font-bold text-black">
                  {holder.amount_percentage.toFixed(2)}%
                </span>
                <div className="w-32 h-2 bg-gray-200 rounded-full mt-1">
                  <div
                    className="h-full bg-blue-500 rounded-full"
                    style={{ width: `${holder.amount_percentage}%` }}
                  />
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
} 