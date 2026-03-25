import * as tauri from "./tauri";

export enum RUN_ENV {
  DESKTOP = "desktop",
}

declare global {
  interface Window {
    __TAURI__?: unknown;
  }
}

export const getRunEnv = (): RUN_ENV => {
  if (typeof window !== "undefined" && window.__TAURI__) {
    return RUN_ENV.DESKTOP;
  }
  return RUN_ENV.DESKTOP;
};

export const invokeTauri = tauri.invokeTauri;

export const logger = tauri.logger;

export type { EventCallback, UnlistenFn } from "./tauri";

export {
  listenDatabaseRestoredTauri, listenFileDropCancelledTauri, listenFileDropHoverTauri,
  listenFileDropTauri, listenMarketSyncCompleteTauri,
  listenMarketSyncStartTauri,
  listenNavigateToRouteTauri, listenPortfolioUpdateCompleteTauri, listenPortfolioUpdateErrorTauri, listenPortfolioUpdateStartTauri, openCsvFileDialogTauri, openDatabaseFileDialogTauri, openFileSaveDialogTauri, openFolderDialogTauri, readBinaryFileTauri
} from "./tauri";

// AI Adapter exports
export {
  streamAiChat,
  listAiThreads,
  getAiThread,
  getAiThreadMessages,
  updateAiThread,
  deleteAiThread,
  addAiThreadTag,
  removeAiThreadTag,
  getAiThreadTags,
  getAiProviders,
  updateAiProviderSettings,
  setDefaultAiProvider,
  listAiModels,
  updateToolResult,
  parseNdjsonStream,
  parseErrorCode,
  ERROR_CODE_MAP,
  setSecret,
  getSecret,
  deleteSecret,
} from "./ai";

export type {
  AiSendMessageRequest,
  AiChatModelConfig,
  AiStreamEvent,
  ChatMessage,
  ChatMessageContent,
  ChatMessagePart,
  AiThread,
  ThreadPage,
  ListThreadsRequest,
  UpdateThreadRequest,
  ToolCall,
  ToolResult,
  UsageStats,
  ChatError,
  AiProvidersResponse,
  AiProviderSetting,
  UpdateProviderSettingsRequest,
  SetDefaultProviderRequest,
  ListModelsResponse,
  UpdateToolResultRequest,
  ModelCapabilities,
  CapabilityInfo,
  ConnectionField,
  ProviderDefaultConfig,
  ModelCatalogEntry,
  ProviderCatalogEntry,
  ModelCapabilityOverrides,
  FetchedModel,
  MergedProvider,
  MergedModel,
} from "./ai";
