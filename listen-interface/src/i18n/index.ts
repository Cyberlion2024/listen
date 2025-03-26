import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { buySellModal } from "./translations/buy-sell-modal";
import { chat } from "./translations/chat";
import { chatHistory } from "./translations/chat-history";
import { gettingStarted } from "./translations/getting-started";
import { layout } from "./translations/layout";
import { pipelineExecution } from "./translations/pipeline-execution";
import { pipelines } from "./translations/pipelines";
import { portfolio } from "./translations/portfolio";
import { priceUpdates } from "./translations/price-updates";
import { recentChats } from "./translations/recent-chats";
import { recommendedQuestions } from "./translations/recommended-questions";
import { settings } from "./translations/settings";
import { shareModal } from "./translations/share-modal";
import { tokenTile } from "./translations/token-tile";
import { toolCalls } from "./translations/tool-calls";
import { toolMessages } from "./translations/tool-messages";
import { version } from "./translations/version";
import { walletAddresses } from "./translations/wallet-addresses";

const resources = {
  en: {
    translation: {
      version: version.en,
      tool_calls: toolCalls.en,
      tool_messages: toolMessages.en,
      getting_started: gettingStarted.en,
      layout: layout.en,
      chat_history: chatHistory.en,
      recent_chats: recentChats.en,
      pipelines: pipelines.en,
      token_tile: tokenTile.en,
      pipeline_execution: pipelineExecution.en,
      price_updates: priceUpdates.en,
      chat: chat.en,
      recommended_questions: recommendedQuestions.en,
      share_modal: shareModal.en,
      settings: settings.en,
      wallet_addresses: walletAddresses.en,
      portfolio: portfolio.en,
      buy_sell_modal: buySellModal.en,
    },
  },
  ar: {
    translation: {
      version: version.ar,
      tool_calls: toolCalls.ar,
      tool_messages: toolMessages.ar,
      getting_started: gettingStarted.ar,
      layout: layout.ar,
      chat_history: chatHistory.ar,
      recent_chats: recentChats.ar,
      pipelines: pipelines.ar,
      token_tile: tokenTile.ar,
      pipeline_execution: pipelineExecution.ar,
      price_updates: priceUpdates.ar,
      chat: chat.ar,
      recommended_questions: recommendedQuestions.ar,
      share_modal: shareModal.ar,
      settings: settings.ar,
      wallet_addresses: walletAddresses.ar,
      portfolio: portfolio.ar,
      buy_sell_modal: buySellModal.ar,
    },
  },
};

const isArabicLocale = (locale: string) => {
  return locale.startsWith("ar-");
};

// Get user's browser locale
const getBrowserLocale = () => {
  const browserLocale = navigator.language;
  if (isArabicLocale(browserLocale)) {
    return "ar";
  }
  return "en";
};

export const savedLanguage =
  localStorage.getItem("language") || getBrowserLocale();

i18n.use(initReactI18next).init({
  resources,
  lng: savedLanguage,
  fallbackLng: "en",
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
