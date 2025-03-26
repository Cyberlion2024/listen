import { ReactNode, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { BsClock } from "react-icons/bs";
import {
  FaChartLine,
  FaCheckCircle,
  FaExclamationTriangle,
  FaSearch,
} from "react-icons/fa";
import { FaXTwitter } from "react-icons/fa6";
import { z } from "zod";
import { CandlestickDataSchema } from "../hooks/types";
import { renderTimestamps } from "../hooks/util";
import { DexScreenerResponseSchema } from "../types/dexscreener";
import { Message, ToolCallSchema, ToolResult } from "../types/message";
import { TokenMetadataSchema } from "../types/metadata";
import {
  JupiterQuoteResponseSchema,
  QuoteResponseSchema,
} from "../types/quote";
import { TweetSchema } from "../types/x";
import { SolanaBalance, SplTokenBalance } from "./Balances";
import { Chart, InnerChart } from "./Chart";
import { ChatMessage } from "./ChatMessage";
import { DexscreenerDisplay } from "./DexscreenerDisplay";
import DropdownMessage from "./DropdownMessage";
import { FetchXPostDisplay } from "./FetchXPostDisplay";
import { JupiterQuoteDisplay } from "./JupiterQuoteDisplay";
import { TransactionLink } from "./PipelineStepContainer";
import { QuoteDisplay } from "./QuoteDisplay";
import { RawTokenMetadataDisplay } from "./RawTokenMetadataDisplay";
import { embedResearchAnchors } from "./ResearchOutput";
import { RiskAnalysisDisplay, RiskAnalysisSchema } from "./RiskDisplay";
import { TopTokensDisplay, TopTokensResponseSchema } from "./TopTokensDisplay";
import LunarCrushDisplay from "./LunarCrushDisplay";

const SplTokenBalanceSchema = z.tuple([z.string(), z.number(), z.string()]);

const formatError = (error: string) => {
  if (error.includes("Invalid param: could not find account")) {
    return "Account not found";
  }
  return error;
};

export const ToolMessage = ({
  toolOutput,
  messages,
  currentMessage,
}: {
  toolOutput: ToolResult;
  messages: Message[];
  currentMessage: Message;
}) => {
  const { t } = useTranslation();

  // Find the corresponding tool call for this tool result
  const matchingToolCall = useMemo(() => {
    if (!toolOutput.id) return null;

    // Find the index of the current message
    const currentIndex = messages.findIndex((m) => m.id === currentMessage.id);
    if (currentIndex === -1) return null;

    // Look backwards through messages to find the matching tool call
    for (let i = currentIndex - 1; i >= 0; i--) {
      const message = messages[i];
      if (message.type === "ToolCall") {
        try {
          const toolCall = ToolCallSchema.parse(JSON.parse(message.message));
          if (toolCall.id === toolOutput.id) {
            return toolCall;
          }
        } catch (e) {
          console.error("Failed to parse tool call:", e);
        }
      }
    }
    return null;
  }, [messages, currentMessage.id, toolOutput.id]);

  if (toolOutput.name === "think") {
    return null;
  }

  if (toolOutput.name === "get_current_time") {
    try {
      const parsed = JSON.parse(toolOutput.result);
      return (
        <div className="text-blue-300 flex items-center gap-1 p-3 text-sm">
          <BsClock /> {new Date(parsed).toLocaleString()}
        </div>
      );
    } catch (e) {
      console.error("Failed to parse current time:", e);
    }
  }

  if (toolOutput.name === "create_advanced_order") {
    try {
      const parsed = JSON.parse(toolOutput.result);
      return (
        <div className="text-green-300 flex items-center gap-1 p-3 text-sm">
          <FaCheckCircle /> {parsed}
        </div>
      );
    } catch (e) {
      console.error("Failed to parse advanced order:", e);
    }
  }

  if (toolOutput.name === "analyze_risk") {
    try {
      const parsed = RiskAnalysisSchema.parse(JSON.parse(toolOutput.result));
      return <RiskAnalysisDisplay riskAnalysis={parsed} />;
    } catch (e) {
      console.error("Failed to parse risk analysis:", e);
    }
  }

  if (toolOutput.name === "fetch_price_action_analysis") {
    try {
      const [mint, interval] = useMemo(() => {
        if (!matchingToolCall) return [null, "30s"];

        try {
          const params = JSON.parse(matchingToolCall.params);
          return [params.mint, params.interval || "30s"];
        } catch (e) {
          console.error("Failed to parse tool call params:", e);
          return [null, "30s"];
        }
      }, [matchingToolCall]);

      if (mint) {
        let parsed = toolOutput.result;
        try {
          parsed = JSON.parse(toolOutput.result);
        } catch (e) {
          console.error("Failed to parse price action analysis:", e);
        }
        return (
          <div className="mb-1">
            <div className="h-[300px] mb-3">
              <Chart mint={mint} interval={interval} />
            </div>
            <DropdownMessage
              title={t("tool_messages.price_action_analysis")}
              message={renderTimestamps(parsed)}
              icon={<FaChartLine />}
            />
          </div>
        );
      }

      return (
        <div className="text-gray-400">
          <ChatMessage message={toolOutput.result} direction="agent" />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse price action analysis:", e);
      return (
        <div className="text-gray-400">
          <ChatMessage message={toolOutput.result} direction="agent" />
        </div>
      );
    }
  }

  if (toolOutput.name === "get_spl_token_balance") {
    try {
      const parsed = SplTokenBalanceSchema.parse(JSON.parse(toolOutput.result));
      return (
        <div className="p-3">
          <SplTokenBalance
            amount={parsed[0]}
            decimals={parsed[1]}
            mint={parsed[2]}
          />
        </div>
      );
    } catch (e) {
      if (toolOutput.result.includes("data: Empty") && matchingToolCall) {
        try {
          const mint = JSON.parse(matchingToolCall.params).mint;
          return (
            <div className="p-3">
              <div className="flex items-center gap-1">
                <SplTokenBalance amount={"0"} decimals={0} mint={mint} />
                <div className="relative group">
                  <span className="text-orange-500 flex items-center gap-1 cursor-help">
                    <FaExclamationTriangle />
                  </span>
                  <div className="absolute left-0 bottom-full mb-2 hidden group-hover:block bg-black/90 text-orange-500 p-2 rounded shadow-lg z-10 max-w-xs break-words w-[200px]">
                    {formatError(toolOutput.result)}
                  </div>
                </div>
              </div>
            </div>
          );
        } catch (e) {
          console.error("Failed to parse tool call:", e);
        }
      }
      console.error("Failed to parse spl token balance:", e);
    }
  }

  if (toolOutput.name === "fetch_x_post") {
    try {
      const parsed = TweetSchema.parse(JSON.parse(toolOutput.result));
      return <FetchXPostDisplay tweet={parsed} />;
    } catch (e) {
      console.error("Failed to parse tweet:", e);
      if (toolOutput.result.includes("No tweet found")) {
        return (
          <div className="p-3">
            <div className="text-orange-500 flex items-center gap-1">
              <FaExclamationTriangle /> {t("tool_messages.no_tweet_found")}
            </div>
          </div>
        );
      }
      return <ChatMessage message={toolOutput.result} direction="agent" />;
    }
  }

  if (toolOutput.name === "search_web") {
    try {
      let message;
      try {
        // Prova prima a parsare come JSON
        message = JSON.parse(toolOutput.result);
      } catch (e) {
        // Se fallisce, usa il risultato come testo semplice
        message = toolOutput.result;
      }
      return (
        <div className="mb-1">
          <DropdownMessage
            title={t("tool_messages.search_web")}
            message={message}
            icon={<FaSearch />}
          />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse web search:", e);
      return (
        <div className="text-red-400 p-3">
          <FaExclamationTriangle className="inline mr-2" />
          {t("tool_messages.search_web_error")}
        </div>
      );
    }
  }
  if (toolOutput.name === "analyze_page_content") {
    try {
      const message = JSON.parse(toolOutput.result);
      return (
        <div className="mb-1">
          <DropdownMessage
            title={t("tool_messages.analyze_page_content")}
            message={message}
            icon={<FaSearch />}
          />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse page content:", e);
    }
  }

  if (toolOutput.name === "search_tweets") {
    try {
      const message = JSON.parse(toolOutput.result);
      const processedMessage = embedResearchAnchors(message);
      return (
        <div className="mb-1">
          <DropdownMessage
            title={t("tool_messages.search_tweets")}
            message={processedMessage}
            icon={<FaXTwitter />}
          />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse tweet:", e);
    }
  }

  if (toolOutput.name === "research_x_profile") {
    try {
      const message = JSON.parse(toolOutput.result);
      const processedMessage = embedResearchAnchors(message);
      return (
        <div className="mb-1">
          <DropdownMessage
            title={t("tool_messages.research_x_profile")}
            message={processedMessage}
            icon={<FaXTwitter />}
          />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse tweet:", e);
      if (toolOutput.result.includes("Account suspended")) {
        return (
          <div className="p-3">
            <div className="text-orange-500 flex items-center gap-1">
              <FaExclamationTriangle /> {t("tool_messages.account_suspended")}
            </div>
          </div>
        );
      }
      if (toolOutput.result.includes("not found")) {
        return (
          <div className="p-3">
            <div className="text-orange-500 flex items-center gap-1">
              <FaExclamationTriangle /> {t("tool_messages.user_does_not_exist")}
            </div>
          </div>
        );
      }
      return <ChatMessage message={toolOutput.result} direction="agent" />;
    }
  }

  if (toolOutput.name === "fetch_token_metadata") {
    try {
      const parsed = TokenMetadataSchema.parse(JSON.parse(toolOutput.result));
      return <RawTokenMetadataDisplay metadata={parsed} />;
    } catch (e) {
      console.error("Failed to parse token metadata:", e);
    }
  }

  if (toolOutput.name === "get_sol_balance") {
    try {
      const parsedLamports = parseInt(toolOutput.result);
      const solanaBalance = parsedLamports / 10 ** 9;
      return (
        <div className="p-3">
          <SolanaBalance solanaBalance={solanaBalance} />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse solana balance:", e);
      return <div>Error parsing solana balance</div>;
    }
  }
  // If it's a dexscreener response, parse and display it
  if (toolOutput.name === "search_on_dex_screener") {
    try {
      const parsed = DexScreenerResponseSchema.parse(
        JSON.parse(toolOutput.result)
      );
      return <DexscreenerDisplay pairs={parsed.pairs} />;
    } catch (e) {
      console.error("Failed to parse dexscreener response:", e);
      return <div>Error parsing DexScreener data</div>;
    }
  }

  if (toolOutput.name === "fetch_candlesticks") {
    try {
      const parsed = CandlestickDataSchema.parse(JSON.parse(toolOutput.result));
      return (
        <div className="h-[300px]">
          <InnerChart data={parsed} />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse candlestick response:", e);
    }
  }

  if (toolOutput.name === "fetch_top_tokens") {
    try {
      const parsed = TopTokensResponseSchema.parse(
        JSON.parse(toolOutput.result)
      );
      return <TopTokensDisplay tokens={parsed} />;
    } catch (e) {
      console.error("Failed to parse top tokens response:", e);
    }
  }
  if (toolOutput.name === "swap") {
    const TxContainer = ({ children }: { children: ReactNode }) => (
      <div className="text-white px-4 py-3 my-2 backdrop-blur-sm overflow-hidden border-b border-[#2d2d2d]">
        {children}
      </div>
    );
    try {
      // TODO standardize this output, not just string but { status: string, transactionHash: string }
      return (
        <TxContainer>
          <div className="mb-2 overflow-hidden">
            <TransactionLink
              status={"Completed"}
              transactionHash={JSON.parse(toolOutput.result)}
              error={null}
            />
          </div>
        </TxContainer>
      );
    } catch (e) {
      console.error("Failed to parse swap response:", e);
      return (
        <TxContainer>
          <div className="mb-2 overflow-hidden">
            <TransactionLink
              status={"Failed"}
              transactionHash={null}
              error={toolOutput.result}
            />
          </div>
        </TxContainer>
      );
    }
  }

  if (toolOutput.name === "analyze_topic") {
    try {
      const topicData = JSON.parse(toolOutput.result);
      return (
        <div className="mb-1">
          <LunarCrushDisplay topicData={topicData} />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse LunarCrush data:", e);
      return (
        <div className="text-gray-400">
          <ChatMessage message={toolOutput.result} direction="agent" />
        </div>
      );
    }
  }

  if (toolOutput.name === "analyze_wallet_concentration") {
    try {
      const walletData = JSON.parse(toolOutput.result);
      
      // Determiniamo la lingua in base a quella usata nella richiesta originale
      // Cerchiamo nel tool call originale per vedere che lingua è stata usata nell'input
      let queryLanguage = 'it'; // Lingua predefinita
      
      // Estraiamo la richiesta originale dal tool call corrispondente, se disponibile
      if (matchingToolCall) {
        try {
          const originalQuery = JSON.parse(matchingToolCall.params).token_address || '';
          // Semplice euristica per determinare la lingua della query
          if (/^[a-zA-Z0-9\s.,:!?]+$/.test(originalQuery)) {
            // Se contiene solo caratteri latini, numeri e punteggiatura comune
            queryLanguage = 'en';
          } else if (/[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF]+/.test(originalQuery)) {
            // Se contiene caratteri arabi
            queryLanguage = 'ar';
          } else if (/[\u4E00-\u9FFF\u3400-\u4DBF\u20000-\u2A6DF\u2A700-\u2B73F]+/.test(originalQuery)) {
            // Se contiene caratteri cinesi
            queryLanguage = 'zh';
          }
        } catch (e) {
          console.error("Errore nel determinare la lingua della query:", e);
        }
      }
      
      // Lingua dell'interfaccia
      const uiLanguage = t('language');
      
      // Se abbiamo entrambe le lingue, scegliamo quella della query
      const preferredLanguage = queryLanguage || uiLanguage;
      
      return (
        <div className="bg-white rounded-lg p-4 my-2 text-black">
          <div className="text-xl font-bold mb-2 border-b border-gray-300 pb-2">
            {preferredLanguage === 'en' 
              ? "Wallet Concentration Analysis" 
              : preferredLanguage === 'ar' 
                ? "تحليل تركيز المحافظ" 
                : "Analisi Concentrazione Wallet"}
          </div>
          
          {walletData.status === "success" ? (
            <div className="space-y-3">
              <div className="flex justify-between items-center">
                <div className="font-bold">
                  {preferredLanguage === 'en' 
                    ? "Token Address:" 
                    : preferredLanguage === 'ar' 
                      ? "عنوان الرمز:" 
                      : "Token Address:"}
                </div>
                <div className="text-sm font-mono">{walletData.token_address}</div>
              </div>
              
              {walletData.max_holder && (
                <div className="bg-gray-100 p-3 rounded">
                  <div className="font-bold mb-1">
                    {preferredLanguage === 'en' 
                      ? "Main Holder:" 
                      : preferredLanguage === 'ar' 
                        ? "المالك الرئيسي:" 
                        : "Holder Principale:"}
                  </div>
                  <div className="flex justify-between">
                    <div className="text-sm font-mono truncate">{walletData.max_holder.address}</div>
                    <div className="font-bold text-red-600">{walletData.max_holder.percentage.toFixed(2)}%</div>
                  </div>
                </div>
              )}
              
              <div className="grid grid-cols-2 gap-4 my-3">
                <div className="bg-gray-100 p-3 rounded">
                  <div className="font-bold mb-1">
                    {preferredLanguage === 'en' 
                      ? "Isolated Wallets:" 
                      : preferredLanguage === 'ar' 
                        ? "المحافظ المعزولة:" 
                        : "Wallet Isolati:"}
                  </div>
                  <div className="flex justify-between">
                    <div>
                      {preferredLanguage === 'en' 
                        ? "Number:" 
                        : preferredLanguage === 'ar' 
                          ? "العدد:" 
                          : "Numero:"}
                    </div>
                    <div className="font-bold">{walletData.holder_risk.isolated_wallets.count}</div>
                  </div>
                  <div className="flex justify-between">
                    <div>
                      {preferredLanguage === 'en' 
                        ? "Total Percentage:" 
                        : preferredLanguage === 'ar' 
                          ? "النسبة الإجمالية:" 
                          : "Percentuale Totale:"}
                    </div>
                    <div className="font-bold">{walletData.holder_risk.isolated_wallets.total_percentage.toFixed(2)}%</div>
                  </div>
                </div>
                <div className="bg-gray-100 p-3 rounded">
                  <div className="font-bold mb-1">
                    {preferredLanguage === 'en' 
                      ? "Linked Wallets:" 
                      : preferredLanguage === 'ar' 
                        ? "المحافظ المرتبطة:" 
                        : "Wallet Collegati:"}
                  </div>
                  <div className="flex justify-between">
                    <div>
                      {preferredLanguage === 'en' 
                        ? "Clusters:" 
                        : preferredLanguage === 'ar' 
                          ? "المجموعات:" 
                          : "Cluster:"}
                    </div>
                    <div className="font-bold">{walletData.holder_risk.linked_wallets.clusters}</div>
                  </div>
                  <div className="flex justify-between">
                    <div>
                      {preferredLanguage === 'en' 
                        ? "Total Percentage:" 
                        : preferredLanguage === 'ar' 
                          ? "النسبة الإجمالية:" 
                          : "Percentuale Totale:"}
                    </div>
                    <div className="font-bold">{walletData.holder_risk.linked_wallets.total_percentage.toFixed(2)}%</div>
                  </div>
                </div>
              </div>
              
              {walletData.holder_risk.linked_wallets.largest_clusters && 
               walletData.holder_risk.linked_wallets.largest_clusters.length > 0 && (
                <div className="my-3">
                  <div className="font-bold mb-2">
                    {preferredLanguage === 'en' 
                      ? "Main Wallet Clusters:" 
                      : preferredLanguage === 'ar' 
                        ? "مجموعات المحافظ الرئيسية:" 
                        : "Principali Cluster di Wallet:"}
                  </div>
                  <div className="space-y-2">
                    {walletData.holder_risk.linked_wallets.largest_clusters.map((cluster: { wallets: number; percentage: number }, idx: number) => (
                      <div key={idx} className="bg-gray-100 p-2 rounded flex justify-between">
                        <div>
                          {preferredLanguage === 'en' 
                            ? `Cluster ${idx+1}: ${cluster.wallets} wallets` 
                            : preferredLanguage === 'ar' 
                              ? `المجموعة ${idx+1}: ${cluster.wallets} محفظة` 
                              : `Cluster ${idx+1}: ${cluster.wallets} wallet`}
                        </div>
                        <div className="font-bold">{cluster.percentage.toFixed(2)}%</div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
              
              <div className="bg-gray-100 p-3 rounded">
                <div className="font-bold mb-1">
                  {preferredLanguage === 'en' 
                    ? "Distribution Metrics:" 
                    : preferredLanguage === 'ar' 
                      ? "مقاييس التوزيع:" 
                      : "Metriche di Distribuzione:"}
                </div>
                <div className="flex justify-between">
                  <div>
                    {preferredLanguage === 'en' 
                      ? "Gini Index:" 
                      : preferredLanguage === 'ar' 
                        ? "مؤشر جيني:" 
                        : "Indice Gini:"}
                  </div>
                  <div className="font-bold">{walletData.holder_risk.distribution.gini_index.toFixed(2)}</div>
                </div>
                <div className="flex justify-between">
                  <div>
                    {preferredLanguage === 'en' 
                      ? "Top 70% Centralization:" 
                      : preferredLanguage === 'ar' 
                        ? "تركيز أعلى 70%:" 
                        : "Centralizzazione Top 70%:"}
                  </div>
                  <div className="font-bold">{walletData.holder_risk.distribution.top70_centralization.toFixed(2)}%</div>
                </div>
              </div>
              
              <div className="mt-4 p-3 rounded font-bold text-center" 
                   style={{
                     backgroundColor: 
                       walletData.holder_risk.risk_level.includes("alto") || 
                       walletData.holder_risk.risk_level.includes("high") || 
                       walletData.holder_risk.risk_level.includes("عالي") 
                         ? "rgba(239, 68, 68, 0.2)" :
                       walletData.holder_risk.risk_level.includes("Moderato") || 
                       walletData.holder_risk.risk_level.includes("moderate") || 
                       walletData.holder_risk.risk_level.includes("متوسط") 
                         ? "rgba(245, 158, 11, 0.2)" :
                       "rgba(34, 197, 94, 0.2)",
                     color:
                       walletData.holder_risk.risk_level.includes("alto") || 
                       walletData.holder_risk.risk_level.includes("high") || 
                       walletData.holder_risk.risk_level.includes("عالي") 
                         ? "rgb(220, 38, 38)" :
                       walletData.holder_risk.risk_level.includes("Moderato") || 
                       walletData.holder_risk.risk_level.includes("moderate") || 
                       walletData.holder_risk.risk_level.includes("متوسط") 
                         ? "rgb(245, 158, 11)" :
                       "rgb(34, 197, 94)"
                   }}>
                {preferredLanguage === 'en' 
                  ? `Risk Level: ${walletData.holder_risk.risk_level}` 
                  : preferredLanguage === 'ar' 
                    ? `مستوى الخطر: ${walletData.holder_risk.risk_level}` 
                    : `Livello di Rischio: ${walletData.holder_risk.risk_level}`}
              </div>
              
              {walletData.distilled_analysis && walletData.distilled_analysis[preferredLanguage] && (
                <div className="mt-4 p-3 text-sm bg-gray-50 rounded border border-gray-200">
                  <ChatMessage 
                    message={walletData.distilled_analysis[preferredLanguage]} 
                    direction="agent" 
                  />
                </div>
              )}
            </div>
          ) : (
            <div className="text-red-500">
              {preferredLanguage === 'en' 
                ? (walletData.message || "Error in wallet analysis") 
                : preferredLanguage === 'ar' 
                  ? (walletData.message || "خطأ في تحليل المحفظة") 
                  : (walletData.message || "Errore nell'analisi dei wallet")}
            </div>
          )}
        </div>
      );
    } catch (e) {
      console.error("Failed to parse Faster100x data:", e);
      return (
        <div className="text-gray-400">
          <ChatMessage message={toolOutput.result} direction="agent" />
        </div>
      );
    }
  }

  if (toolOutput.name === "get_quote") {
    try {
      // Clean up the result string in case it has extra escaping
      let resultData = toolOutput.result;

      // Sometimes quotes get double-stringified, check for that
      if (resultData.startsWith('"') && resultData.endsWith('"')) {
        try {
          // Try to parse once to remove outer quotes if double-stringified
          resultData = JSON.parse(resultData);
        } catch (e) {
          // If this fails, keep the original string
          resultData = toolOutput.result;
        }
      }

      try {
        // Parse the data to an object
        const parsedData =
          typeof resultData === "string" ? JSON.parse(resultData) : resultData;

        // First try Jupiter quote schema
        try {
          const jupiterQuote = JupiterQuoteResponseSchema.parse(parsedData);
          return <JupiterQuoteDisplay quote={jupiterQuote} />;
        } catch (jupiterError) {
          console.error("Jupiter quote validation failed:", jupiterError);

          // Then try regular quote schema
          try {
            const quote = QuoteResponseSchema.parse(parsedData);
            return <QuoteDisplay quote={quote} />;
          } catch (quoteError) {
            console.error("Regular quote validation failed:", quoteError);
            throw new Error("Failed to validate quote with either schema");
          }
        }
      } catch (parseError) {
        console.error("JSON parse error:", parseError);
        throw parseError;
      }
    } catch (e) {
      console.error("Quote processing failed:", e);

      return (
        <div className="bg-blue-900/20 text-blue-300 rounded-lg px-4 py-3 my-2 backdrop-blur-sm border border-opacity-20 border-blue-500">
          <p className="text-red-400 break-words">
            Failed to parse quote data:{" "}
            {e instanceof Error ? e.message : "Unknown error"}
          </p>
          <details>
            <summary className="cursor-pointer text-sm">
              View raw quote data
            </summary>
            <pre className="text-xs mt-2 overflow-x-auto p-2 bg-gray-800 rounded break-words whitespace-pre-wrap">
              {typeof toolOutput.result === "string"
                ? toolOutput.result
                : JSON.stringify(toolOutput.result, null, 2)}
            </pre>
          </details>
        </div>
      );
    }
  }

  // Default tool output display
  return (
    <div className="text-blue-300 rounded-lg px-4 py-3 my-2 backdrop-blur-sm border border-opacity-20 border-blue-500 overflow-hidden">
      <ChatMessage message={toolOutput.result} direction="incoming" />
    </div>
  );
};
