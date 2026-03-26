/**
 * AI Provider Settings Component
 *
 * Displays and manages AI provider configurations with full feature support.
 */

import { useState, useMemo, useEffect, useRef } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import {
  Check,
  CircleAlert,
  Plus,
  X,
  Settings,
  Sparkles,
  ExternalLink,
  Star,
  AlertTriangle,
  Eye,
  EyeOff,
  RefreshCw,
  Loader2 as Spinner,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { Badge } from "@/components/ui/badge";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { Checkbox } from "@/components/ui/checkbox";
import { Switch } from "@/components/ui/switch";
import { toast } from "sonner";
import { cn } from "@/lib/utils";

import {
  getAiProviders,
  updateAiProviderSettings,
  listAiModels,
  setSecret,
  getSecret,
  deleteSecret,
  type AiProviderSetting,
  type ModelCapabilityOverrides,
} from "@/adapters/ai";
import { ProviderIcon } from "./provider-icons";

// Data access options for tools allowlist
const DATA_ACCESS_OPTIONS = [
  { toolId: "get_accounts", label: "Accounts", description: "Account names, types, and balances" },
  { toolId: "get_holdings", label: "Holdings", description: "Current positions and their values" },
  {
    toolId: "search_activities",
    label: "Transactions",
    description: "Past transactions and activities",
  },
  {
    toolId: "get_performance",
    label: "Performance",
    description: "Returns and performance metrics",
  },
  { toolId: "get_income", label: "Income", description: "Income summary and breakdown" },
  { toolId: "get_goals", label: "Goals", description: "Investment goals and progress" },
  {
    toolId: "get_asset_allocation",
    label: "Allocation",
    description: "Portfolio allocation breakdown",
  },
  { toolId: "get_valuation_history", label: "History", description: "Portfolio value over time" },
];

interface ProviderSettingsCardProps {
  provider: MergedProvider;
  isLast: boolean;
  onToggleEnabled: (enabled: boolean) => void;
  onSaveApiKey: (apiKey: string) => void;
  onDeleteApiKey: () => void;
  onRevealApiKey: () => Promise<string | null>;
  onCustomUrlChange?: (url: string) => void;
  onSetFavoriteModels?: (modelIds: string[]) => void;
  onSetCapabilityOverride?: (modelId: string, overrides: ModelCapabilityOverrides | null) => void;
  onToolsAllowlistChange?: (tools: string[] | null) => void;
  // Model fetching props
  modelComboboxOpen?: boolean;
  onModelComboboxOpenChange?: (open: boolean) => void;
  fetchedModels?: FetchedModel[];
  isFetchingModels?: boolean;
  fetchModelsError?: string | null;
  onRefreshModels?: () => void;
}

interface MergedProvider {
  id: string;
  name: string;
  description: string;
  type: "api" | "local";
  icon: string;
  enabled: boolean;
  isDefault: boolean;
  hasApiKey: boolean;
  priority: number;
  customUrl?: string;
  documentationUrl?: string;
  supportsModelListing: boolean;
  connectionFields: Array<{
    key: string;
    label: string;
    type: "text" | "password";
    placeholder: string;
    required: boolean;
    helpUrl?: string;
  }>;
  models: MergedModel[];
  favoriteModels?: string[];
  selectedModel?: string;
  modelCapabilityOverrides: Record<string, ModelCapabilityOverrides>;
  toolsAllowlist?: string[] | null;
}

interface MergedModel {
  id: string;
  name: string;
  capabilities?: ModelCapabilityOverrides;
  isCatalog: boolean;
}

interface FetchedModel {
  id: string;
  name: string;
  description?: string;
}

export function ProviderSettingsCard({
  provider,
  isLast,
  onToggleEnabled,
  onSaveApiKey,
  onDeleteApiKey,
  onRevealApiKey,
  onCustomUrlChange,
  onSetFavoriteModels,
  onSetCapabilityOverride,
  onToolsAllowlistChange,
  modelComboboxOpen: controlledComboboxOpen,
  onModelComboboxOpenChange,
  fetchedModels: externalFetchedModels,
  isFetchingModels: externalIsFetchingModels,
  fetchModelsError: externalFetchModelsError,
  onRefreshModels,
}: ProviderSettingsCardProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [showApiKey, setShowApiKey] = useState(false);
  const [apiKeyValue, setApiKeyValue] = useState("");
  const [isLoadingKey, setIsLoadingKey] = useState(false);
  const [hasLoadedKey, setHasLoadedKey] = useState(false);
  const [customUrlValue, setCustomUrlValue] = useState(provider.customUrl ?? "");
  const [selectedModelForConfig, setSelectedModelForConfig] = useState<string | null>(null);
  const hasAutoSelectedRef = useRef(false);

  // Support both controlled and uncontrolled combobox state
  const [internalComboboxOpen, setInternalComboboxOpen] = useState(false);
  const modelComboboxOpen = controlledComboboxOpen ?? internalComboboxOpen;
  const setModelComboboxOpen = onModelComboboxOpenChange ?? setInternalComboboxOpen;

  // Use external fetched models if provided
  const fetchedModels = externalFetchedModels ?? [];
  const isFetchingModels = externalIsFetchingModels ?? false;
  const fetchError = externalFetchModelsError ?? null;

  // Check if provider supports custom base URL
  const supportsCustomUrl = provider.connectionFields?.some(
    (field) => field.key === "baseUrl" || field.key === "customUrl",
  );
  const customUrlField = provider.connectionFields?.find(
    (field) => field.key === "baseUrl" || field.key === "customUrl",
  );

  // Combine catalog models with fetched models and saved favorites
  const allModels = useMemo(() => {
    const seenIds = new Set<string>();
    const combined: (MergedModel | (FetchedModel & { isFetched: true }))[] = [];

    // First add catalog models
    for (const model of provider.models) {
      combined.push(model);
      seenIds.add(model.id);
    }

    // Add fetched models that aren't in catalog
    for (const fetched of fetchedModels) {
      if (!seenIds.has(fetched.id)) {
        combined.push({
          ...fetched,
          isFetched: true,
        } as FetchedModel & { isFetched: true });
        seenIds.add(fetched.id);
      }
    }

    // Add any saved favorite models that aren't already in the list
    for (const favoriteId of provider.favoriteModels || []) {
      if (!seenIds.has(favoriteId)) {
        combined.push({
          id: favoriteId,
          name: favoriteId,
          isFetched: true,
        } as FetchedModel & { isFetched: true });
        seenIds.add(favoriteId);
      }
    }

    return combined;
  }, [provider.models, fetchedModels, provider.favoriteModels]);

  // Get enabled models (favorites) with full model info
  const enabledModels = useMemo(() => {
    const favoriteIds = provider.favoriteModels || [];
    return allModels.filter((m) => favoriteIds.includes(m.id));
  }, [provider.favoriteModels, allModels]);

  const handleRevealApiKey = async () => {
    if (hasLoadedKey) {
      setShowApiKey(!showApiKey);
      return;
    }

    setIsLoadingKey(true);
    try {
      const key = await onRevealApiKey();
      if (key) {
        setApiKeyValue(key);
      }
    } finally {
      setIsLoadingKey(false);
      setHasLoadedKey(true);
      setShowApiKey(true);
    }
  };

  const handleSaveApiKey = () => {
    if (apiKeyValue && apiKeyValue.trim() !== "") {
      onSaveApiKey(apiKeyValue);
    } else {
      onDeleteApiKey();
    }
  };

  // Auto-select recommended models on initial mount if no models are selected
  useEffect(() => {
    if (isOpen && !hasAutoSelectedRef.current && onSetFavoriteModels && provider.enabled) {
      hasAutoSelectedRef.current = true;

      if (!provider.favoriteModels || provider.favoriteModels.length === 0) {
        const recommendedModelIds = provider.models.filter((m) => m.isCatalog).map((m) => m.id);
        if (recommendedModelIds.length > 0) {
          onSetFavoriteModels(recommendedModelIds);
        }
      }
    }
  }, [isOpen, provider.enabled, provider.favoriteModels, provider.models, onSetFavoriteModels]);

  const handleToggleFavorite = (modelId: string) => {
    if (!onSetFavoriteModels) return;

    const currentFavorites = provider.favoriteModels || [];
    const newFavorites = currentFavorites.includes(modelId)
      ? currentFavorites.filter((id) => id !== modelId)
      : [...currentFavorites, modelId];
    onSetFavoriteModels(newFavorites);
  };

  const handleCapabilityChange = (
    modelId: string,
    capability: "tools" | "thinking" | "vision",
    value: boolean,
  ) => {
    if (!onSetCapabilityOverride) return;

    const existingOverrides = provider.modelCapabilityOverrides[modelId] || {};
    const newOverrides: ModelCapabilityOverrides = {
      ...existingOverrides,
      [capability]: value,
    };
    onSetCapabilityOverride(modelId, newOverrides);
  };

  // Handle tool allowlist toggle
  const handleToolToggle = (toolId: string, enabled: boolean) => {
    if (!onToolsAllowlistChange) return;

    const currentAllowlist = provider.toolsAllowlist;
    const allToolIds = DATA_ACCESS_OPTIONS.map((opt) => opt.toolId);

    if (currentAllowlist === null || currentAllowlist === undefined) {
      // Currently all tools enabled (null = all). If disabling one, create allowlist with all except this one.
      if (!enabled) {
        const newAllowlist = allToolIds.filter((id) => id !== toolId);
        onToolsAllowlistChange(newAllowlist);
      }
    } else {
      // We have an explicit allowlist
      if (enabled) {
        // Add tool to allowlist
        const newAllowlist = [...currentAllowlist, toolId];
        onToolsAllowlistChange(newAllowlist);
      } else {
        // Remove tool from allowlist
        const newAllowlist = currentAllowlist.filter((id) => id !== toolId);
        onToolsAllowlistChange(newAllowlist);
      }
    }
  };

  // Check if a tool is enabled
  const isToolEnabled = (toolId: string): boolean => {
    const allowlist = provider.toolsAllowlist;
    // null/undefined means all tools are enabled
    if (allowlist === null || allowlist === undefined) return true;
    return allowlist.includes(toolId);
  };

  return (
    <Collapsible open={isOpen} onOpenChange={setIsOpen}>
      <div className={cn("hover:bg-accent/30 transition-colors", !isLast && "border-b")}>
        {/* Main row */}
        <div className="flex items-center gap-4 px-4 py-3">
          {/* Icon */}
          <div className="bg-muted flex h-9 w-9 shrink-0 items-center justify-center rounded-lg">
            <ProviderIcon name={provider.icon} size={20} className="text-muted-foreground" />
          </div>

          {/* Name and description */}
          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-center gap-2">
              <span className="font-medium">{provider.name}</span>
              {provider.isDefault && (
                <Badge variant="secondary" className="h-5 px-1.5 text-[10px] font-normal">
                  Default
                </Badge>
              )}
              {provider.enabled && !provider.hasApiKey && provider.type === "api" && (
                <Badge
                  variant="outline"
                  className="border-warning/20 bg-warning/10 text-warning shrink-0 text-xs"
                >
                  <AlertTriangle className="mr-1 h-3 w-3" />
                  API Key Required
                </Badge>
              )}
            </div>
            <p className="text-muted-foreground mt-0.5 text-xs">{provider.description}</p>
          </div>

          {/* Controls */}
          <div className="flex shrink-0 items-center gap-2">
            <CollapsibleTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="text-muted-foreground hover:text-foreground h-8 w-8"
              >
                <Settings className="h-4 w-4" />
              </Button>
            </CollapsibleTrigger>
            <Switch
              id={`${provider.id}-enabled`}
              checked={provider.enabled}
              onCheckedChange={onToggleEnabled}
              className="data-[state=checked]:bg-success"
            />
          </div>
        </div>

        {/* Expandable settings */}
        <CollapsibleContent>
          <div className="border-t px-4 py-5">
            <div className="space-y-5">
              {/* API Key Section (only for API providers) */}
              {provider.type === "api" && (
                <div className="bg-muted/40 rounded-lg p-4">
                  <div className="space-y-3">
                    <div className="flex items-center justify-between">
                      <Label htmlFor={`apikey-${provider.id}`} className="text-sm font-medium">
                        API Key
                      </Label>
                      {provider.documentationUrl && (
                        <a
                          href={provider.documentationUrl}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-muted-foreground hover:text-foreground inline-flex items-center gap-1 text-xs transition-colors"
                        >
                          Get API key
                          <ExternalLink className="h-3 w-3" />
                        </a>
                      )}
                    </div>
                    <div className="flex items-center gap-2">
                      <div className="relative flex-1">
                        <Input
                          id={`apikey-${provider.id}`}
                          type={showApiKey ? "text" : "password"}
                          value={
                            hasLoadedKey || apiKeyValue
                              ? apiKeyValue
                              : provider.hasApiKey
                                ? "••••••••••••••••••••••••"
                                : ""
                          }
                          onChange={(e) => setApiKeyValue(e.target.value)}
                          placeholder={provider.hasApiKey ? "" : "Enter API key"}
                          className="bg-background pr-9 font-mono text-sm"
                          readOnly={!hasLoadedKey && provider.hasApiKey}
                        />
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon"
                          className="absolute right-0 top-0 h-full w-9 hover:bg-transparent"
                          onClick={handleRevealApiKey}
                          disabled={isLoadingKey}
                          aria-label={showApiKey ? "Hide API key" : "Show API key"}
                        >
                          {isLoadingKey ? (
                            <Spinner className="h-4 w-4 animate-spin" />
                          ) : showApiKey ? (
                            <EyeOff className="h-4 w-4" />
                          ) : (
                            <Eye className="h-4 w-4" />
                          )}
                        </Button>
                      </div>
                      <Button
                        onClick={handleSaveApiKey}
                        size="default"
                        className="shrink-0"
                        disabled={!hasLoadedKey && provider.hasApiKey}
                      >
                        Save
                      </Button>
                    </div>
                  </div>
                </div>
              )}

              {/* Model Selection Section */}
              {onSetFavoriteModels && (
                <div className="bg-muted/40 rounded-lg p-4">
                  <div className="space-y-3">
                    {/* Header with Add button */}
                    <div className="flex items-center justify-between">
                      <Label className="text-sm font-medium">Models</Label>
                      <div className="flex items-center gap-2">
                        {provider.supportsModelListing && onRefreshModels && (
                          <Button
                            variant="ghost"
                            size="sm"
                            className="text-muted-foreground hover:text-foreground h-7 px-2 text-xs"
                            onClick={onRefreshModels}
                            disabled={
                              isFetchingModels || (!provider.hasApiKey && provider.type === "api")
                            }
                          >
                            {isFetchingModels ? (
                              <Spinner className="h-3 w-3 animate-spin" />
                            ) : (
                              <RefreshCw className="h-3 w-3" />
                            )}
                          </Button>
                        )}
                        <Popover open={modelComboboxOpen} onOpenChange={setModelComboboxOpen}>
                          <PopoverTrigger asChild>
                            <Button variant="outline" size="sm" className="h-7 gap-1 text-xs">
                              <Plus className="h-3 w-3" />
                              Add
                            </Button>
                          </PopoverTrigger>
                          <PopoverContent className="w-80 p-0" align="end">
                            <Command>
                              <CommandInput placeholder="Search models..." className="h-9" />
                              <CommandList>
                                <CommandEmpty>
                                  {isFetchingModels ? (
                                    <div className="flex items-center justify-center gap-2 py-2">
                                      <Spinner className="h-4 w-4 animate-spin" />
                                      <span>Loading...</span>
                                    </div>
                                  ) : (
                                    "No models found."
                                  )}
                                </CommandEmpty>
                                {/* Recommended models */}
                                <CommandGroup heading="Recommended">
                                  {allModels
                                    .filter((m) => "isCatalog" in m && m.isCatalog)
                                    .map((model) => {
                                      const isEnabled = provider.favoriteModels?.includes(model.id);
                                      const capabilities =
                                        "capabilities" in model ? model.capabilities : null;
                                      return (
                                        <CommandItem
                                          key={model.id}
                                          value={`${model.id} ${model.name ?? ""}`}
                                          onSelect={() => {
                                            handleToggleFavorite(model.id);
                                          }}
                                          className="flex items-center justify-between"
                                        >
                                          <span className="truncate">{model.name ?? model.id}</span>
                                          <div className="flex items-center gap-1">
                                            {capabilities?.tools && (
                                              <Badge
                                                variant="secondary"
                                                className="h-4 px-1 text-[9px]"
                                              >
                                                T
                                              </Badge>
                                            )}
                                            {capabilities?.vision && (
                                              <Badge
                                                variant="secondary"
                                                className="h-4 px-1 text-[9px]"
                                              >
                                                V
                                              </Badge>
                                            )}
                                            {isEnabled && <Check className="h-4 w-4" />}
                                          </div>
                                        </CommandItem>
                                      );
                                    })}
                                </CommandGroup>
                                {/* Other available models */}
                                {allModels.filter((m) => !("isCatalog" in m && m.isCatalog))
                                  .length > 0 && (
                                  <CommandGroup heading="Other Available">
                                    {allModels
                                      .filter((m) => !("isCatalog" in m && m.isCatalog))
                                      .map((model) => {
                                        const isEnabled = provider.favoriteModels?.includes(
                                          model.id,
                                        );
                                        return (
                                          <CommandItem
                                            key={model.id}
                                            value={`${model.id} ${model.name ?? ""}`}
                                            onSelect={() => {
                                              handleToggleFavorite(model.id);
                                            }}
                                            className="flex items-center justify-between"
                                          >
                                            <span className="truncate">
                                              {model.name ?? model.id}
                                            </span>
                                            {isEnabled && <Check className="h-4 w-4" />}
                                          </CommandItem>
                                        );
                                      })}
                                  </CommandGroup>
                                )}
                              </CommandList>
                            </Command>
                          </PopoverContent>
                        </Popover>
                      </div>
                    </div>

                    {/* Model list */}
                    <div className="bg-background rounded-md border">
                      {enabledModels.length === 0 ? (
                        <div className="text-muted-foreground flex items-center justify-center py-6 text-sm">
                          No models selected. Click &quot;Add&quot; to add models.
                        </div>
                      ) : (
                        <div className="divide-y">
                          {enabledModels.map((model) => {
                            const capabilities =
                              "capabilities" in model
                                ? model.capabilities
                                : provider.modelCapabilityOverrides[model.id];
                            const isRecommended = "isCatalog" in model && model.isCatalog;
                            const needsConfig = !isRecommended && !capabilities;
                            const isSelected = selectedModelForConfig === model.id;

                            return (
                              <div
                                key={model.id}
                                className={cn(
                                  "flex cursor-pointer items-center justify-between px-3 py-2 transition-colors",
                                  isSelected ? "bg-accent" : "hover:bg-muted/50",
                                )}
                                onClick={() =>
                                  setSelectedModelForConfig(isSelected ? null : model.id)
                                }
                              >
                                <div className="flex min-w-0 items-center gap-2">
                                  <span className="truncate text-sm">{model.name ?? model.id}</span>
                                  {isRecommended && (
                                    <Star className="text-warning h-3.5 w-3.5 shrink-0 fill-current" />
                                  )}
                                </div>
                                <div className="flex shrink-0 items-center gap-2">
                                  {/* Capability badges */}
                                  {capabilities?.tools && (
                                    <Badge variant="secondary" className="h-5 px-1.5 text-[10px]">
                                      Tools
                                    </Badge>
                                  )}
                                  {capabilities?.vision && (
                                    <Badge variant="secondary" className="h-5 px-1.5 text-[10px]">
                                      Vision
                                    </Badge>
                                  )}
                                  {capabilities?.thinking && (
                                    <Badge variant="secondary" className="h-5 px-1.5 text-[10px]">
                                      Thinking
                                    </Badge>
                                  )}
                                  {needsConfig && (
                                    <Badge
                                      variant="outline"
                                      className="border-warning/50 text-warning h-5 px-1.5 text-[10px]"
                                    >
                                      <AlertTriangle className="mr-1 h-3 w-3" />
                                      Config
                                    </Badge>
                                  )}
                                  {/* Remove button */}
                                  <Button
                                    variant="ghost"
                                    size="icon"
                                    className="text-muted-foreground hover:text-destructive h-6 w-6"
                                    onClick={(e) => {
                                      e.stopPropagation();
                                      handleToggleFavorite(model.id);
                                      if (isSelected) setSelectedModelForConfig(null);
                                    }}
                                  >
                                    <X className="h-3.5 w-3.5" />
                                  </Button>
                                </div>
                              </div>
                            );
                          })}
                        </div>
                      )}
                    </div>

                    {/* Capability config for selected model */}
                    {selectedModelForConfig &&
                      onSetCapabilityOverride &&
                      (() => {
                        const model = enabledModels.find((m) => m.id === selectedModelForConfig);
                        if (!model) return null;
                        const isRecommended = "isCatalog" in model && model.isCatalog;
                        const defaultCapabilities = { tools: false, thinking: false, vision: false };
                        const capabilities =
                          isRecommended && "capabilities" in model
                            ? { ...defaultCapabilities, ...model.capabilities }
                            : { ...defaultCapabilities, ...(provider.modelCapabilityOverrides?.[model.id] || {}) };

                        return (
                          <div className="bg-background rounded-md border p-3">
                            <div className="mb-3 flex items-center justify-between">
                              <p className="text-sm font-medium">{model.name ?? model.id}</p>
                              {isRecommended && (
                                <Badge variant="secondary" className="text-xs">
                                  Recommended
                                </Badge>
                              )}
                            </div>
                            <div className="flex flex-wrap gap-4">
                              <label className="flex items-center gap-2 text-sm">
                                <Checkbox
                                  checked={capabilities.tools ?? false}
                                  onCheckedChange={(checked) =>
                                    handleCapabilityChange(model.id, "tools", checked === true)
                                  }
                                  disabled={isRecommended}
                                />
                                Tools
                              </label>
                              <label className="flex items-center gap-2 text-sm">
                                <Checkbox
                                  checked={capabilities.vision ?? false}
                                  onCheckedChange={(checked) =>
                                    handleCapabilityChange(model.id, "vision", checked === true)
                                  }
                                  disabled={isRecommended}
                                />
                                Vision
                              </label>
                              <label className="flex items-center gap-2 text-sm">
                                <Checkbox
                                  checked={capabilities.thinking ?? false}
                                  onCheckedChange={(checked) =>
                                    handleCapabilityChange(model.id, "thinking", checked === true)
                                  }
                                  disabled={isRecommended}
                                />
                                Thinking
                              </label>
                            </div>
                            {isRecommended && (
                              <p className="text-muted-foreground mt-2 text-xs">
                                Capabilities are preset for recommended models.
                              </p>
                            )}
                          </div>
                        );
                      })()}

                    {/* Fetch error */}
                    {fetchError && <p className="text-destructive text-xs">{fetchError}</p>}
                  </div>
                </div>
              )}

              {/* Custom Base URL Section */}
              {supportsCustomUrl && customUrlField && onCustomUrlChange && (
                <div className="bg-muted/40 rounded-lg p-4">
                  <div className="space-y-2">
                    <div className="flex items-center justify-between">
                      <Label htmlFor={`baseurl-${provider.id}`} className="text-sm font-medium">
                        {customUrlField.label}
                      </Label>
                      {customUrlField.helpUrl && (
                        <a
                          href={customUrlField.helpUrl}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-muted-foreground hover:text-foreground inline-flex items-center gap-1 text-xs transition-colors"
                        >
                          Learn more
                          <ExternalLink className="h-3 w-3" />
                        </a>
                      )}
                    </div>
                    <div className="flex items-center gap-2">
                      <Input
                        id={`baseurl-${provider.id}`}
                        type="url"
                        value={customUrlValue}
                        onChange={(e) => setCustomUrlValue(e.target.value)}
                        placeholder={customUrlField.placeholder}
                        className="bg-background flex-1 font-mono text-sm"
                      />
                      <Button
                        onClick={() => onCustomUrlChange(customUrlValue)}
                        size="default"
                        className="shrink-0"
                      >
                        Save
                      </Button>
                    </div>
                  </div>
                </div>
              )}

              {/* Data Access Section */}
              {onToolsAllowlistChange && (
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label className="text-sm font-medium">Data Access</Label>
                    <span className="text-muted-foreground text-xs">
                      What data the AI can access
                    </span>
                  </div>
                  <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
                    {DATA_ACCESS_OPTIONS.map((option) => {
                      const isEnabled = isToolEnabled(option.toolId);
                      return (
                        <button
                          key={option.toolId}
                          type="button"
                          onClick={() => handleToolToggle(option.toolId, !isEnabled)}
                          className={cn(
                            "flex items-start gap-2.5 rounded-lg border p-3 text-left transition-all",
                            isEnabled
                              ? "border-primary/30 bg-primary/5"
                              : "bg-muted/40 hover:bg-muted/60 border-transparent",
                          )}
                        >
                          <div
                            className={cn(
                              "mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-sm border transition-colors",
                              isEnabled
                                ? "border-primary bg-primary text-primary-foreground"
                                : "border-muted-foreground/30",
                            )}
                          >
                            {isEnabled && <Check className="h-3 w-3" />}
                          </div>
                          <div className="min-w-0 flex-1">
                            <span className="text-sm font-medium">{option.label}</span>
                            <p className="text-muted-foreground mt-0.5 text-xs leading-tight">
                              {option.description}
                            </p>
                          </div>
                        </button>
                      );
                    })}
                  </div>
                </div>
              )}
            </div>
          </div>
        </CollapsibleContent>
      </div>
    </Collapsible>
  );
}

function ProviderSettingsCardWrapper({
  provider,
  isLast,
  onToggleEnabled,
  onCustomUrlChange,
  onSetFavoriteModels,
  onSetCapabilityOverride,
  onToolsAllowlistChange,
}: {
  provider: AiProviderSetting;
  isLast: boolean;
  onToggleEnabled: (enabled: boolean) => void;
  onCustomUrlChange: (url: string) => void;
  onSetFavoriteModels: (modelIds: string[]) => void;
  onSetCapabilityOverride: (modelId: string, overrides: ModelCapabilityOverrides | null) => void;
  onToolsAllowlistChange: (tools: string[] | null) => void;
}) {
  const [modelComboboxOpen, setModelComboboxOpen] = useState(false);

  const secretKey = `ai_${provider.id}`;

  const {
    data: modelsResponse,
    isLoading: isFetchingModels,
    error: fetchModelsError,
    refetch: refetchModels,
  } = useQuery({
    queryKey: ["ai_models", provider.id],
    queryFn: () => listAiModels(provider.id),
    enabled: modelComboboxOpen && (provider.hasApiKey || !provider.requiresApiKey),
  });

  const fetchedModels = modelsResponse?.models ?? [];

  const setApiKey = useMutation({
    mutationFn: async (apiKey: string) => {
      await setSecret(secretKey, apiKey);
    },
    onSuccess: () => {
      toast.success("API key saved");
    },
    onError: (error) => {
      toast.error(error instanceof Error ? error.message : "Failed to save API key");
    },
  });

  const deleteApiKey = useMutation({
    mutationFn: async () => {
      await deleteSecret(secretKey);
    },
    onSuccess: () => {
      toast.success("API key removed");
    },
    onError: (error) => {
      toast.error(error instanceof Error ? error.message : "Failed to remove API key");
    },
  });

  const revealApiKey = async (): Promise<string | null> => {
    return getSecret(secretKey);
  };

  // Convert AiProviderSetting to MergedProvider
  const mergedProvider: MergedProvider = {
    id: provider.id,
    name: provider.name,
    description: provider.description ?? "",
    type: provider.type ?? (provider.requiresApiKey ? "api" : "local"),
    icon: provider.icon ?? "LogoAnthropic",
    enabled: provider.enabled,
    isDefault: provider.isDefault ?? false,
    hasApiKey: provider.hasApiKey,
    priority: provider.priority,
    customUrl: provider.url ?? undefined,
    documentationUrl: provider.documentationUrl ?? undefined,
    supportsModelListing: provider.id !== "ollama", // Example: Ollama doesn't support listing
    connectionFields: provider.connectionFields ?? [],
    models: provider.models ?? [],
    favoriteModels: provider.favoriteModels ?? [],
    selectedModel: provider.selectedModel,
    modelCapabilityOverrides: provider.modelCapabilityOverrides ?? {},
    toolsAllowlist: provider.toolsAllowlist,
  };

  return (
    <ProviderSettingsCard
      provider={mergedProvider}
      isLast={isLast}
      onToggleEnabled={onToggleEnabled}
      onSaveApiKey={(apiKey) => setApiKey.mutate(apiKey)}
      onDeleteApiKey={() => deleteApiKey.mutate()}
      onRevealApiKey={revealApiKey}
      onCustomUrlChange={onCustomUrlChange}
      onSetFavoriteModels={onSetFavoriteModels}
      onSetCapabilityOverride={onSetCapabilityOverride}
      onToolsAllowlistChange={onToolsAllowlistChange}
      modelComboboxOpen={modelComboboxOpen}
      onModelComboboxOpenChange={setModelComboboxOpen}
      fetchedModels={fetchedModels}
      isFetchingModels={isFetchingModels}
      fetchModelsError={fetchModelsError?.message ?? null}
      onRefreshModels={() => refetchModels()}
    />
  );
}

export function AiProviderSettings() {
  const { data: providersData, isLoading, error, refetch } = useQuery({
    queryKey: ["ai-providers"],
    queryFn: getAiProviders,
  });

  const { mutate: updateSettings } = useMutation({
    mutationFn: async (request: { providerId: string; [key: string]: unknown }) => {
      await updateAiProviderSettings(request as any);
    },
    onSuccess: () => {
      refetch();
    },
  });

  const providers = providersData?.providers || [];
  const sortedProviders = [...providers].sort((a, b) => a.priority - b.priority);

  const handleToggleEnabled = (providerId: string, enabled: boolean) => {
    updateSettings({ providerId, enabled });
  };

  const handleCustomUrlChange = (providerId: string, customUrl: string) => {
    updateSettings({ providerId, customUrl });
  };

  const handleSetFavoriteModels = (providerId: string, modelIds: string[]) => {
    updateSettings({ providerId, favoriteModels: modelIds });
  };

  const handleSetCapabilityOverride = (
    providerId: string,
    modelId: string,
    overrides: ModelCapabilityOverrides | null,
  ) => {
    updateSettings({
      providerId,
      modelCapabilityOverride: { modelId, overrides: overrides ?? undefined },
    });
  };

  const handleToolsAllowlistChange = (providerId: string, tools: string[] | null) => {
    updateSettings({ providerId, toolsAllowlist: tools });
  };

  if (isLoading) {
    return (
      <div className="space-y-4">
        {[1, 2, 3].map((i) => (
          <Card key={i} className="h-32 animate-pulse" />
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <div className="border-destructive/20 bg-destructive/5 rounded-lg border p-6">
        <div className="flex items-start gap-3">
          <CircleAlert className="text-destructive mt-0.5 h-5 w-5 shrink-0" />
          <div className="space-y-2">
            <h3 className="text-destructive font-medium">Failed to load AI providers</h3>
            <p className="text-muted-foreground text-sm">{error.message}</p>
            <Button variant="outline" size="sm" onClick={() => refetch()} className="mt-2">
              <RefreshCw className="mr-2 h-4 w-4" />
              Retry
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {sortedProviders.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <Sparkles className="mb-4 size-12 text-muted-foreground/50" />
            <h3 className="text-lg font-semibold">No AI Providers Available</h3>
            <p className="text-muted-foreground mt-2 text-sm">
              AI providers will appear here once configured. Check back later or contact support.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="overflow-hidden rounded-lg border">
          {sortedProviders.map((provider, index, arr) => (
            <ProviderSettingsCardWrapper
              key={provider.id}
              provider={provider}
              isLast={index === arr.length - 1}
              onToggleEnabled={(enabled) => handleToggleEnabled(provider.id, enabled)}
              onCustomUrlChange={(url) => handleCustomUrlChange(provider.id, url)}
              onSetFavoriteModels={(modelIds) => handleSetFavoriteModels(provider.id, modelIds)}
              onSetCapabilityOverride={(modelId, overrides) =>
                handleSetCapabilityOverride(provider.id, modelId, overrides)
              }
              onToolsAllowlistChange={(tools) => handleToolsAllowlistChange(provider.id, tools)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
