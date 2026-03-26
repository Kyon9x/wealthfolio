/**
 * Provider Icons Component
 * Renders icons for different AI providers using lucide-react icons
 */

import { LucideIcon } from "lucide-react";
import { Icons } from "@/ui/components/ui/icons";

export interface ProviderIconProps {
  name: string;
  size?: number;
  className?: string;
}

const ICON_MAP: Record<string, LucideIcon> = {
  LogoAnthropic: Icons.Bot,
  LogoGoogle: Icons.Sparkles,
  LogoGroq: Icons.Zap,
  LogoOllama: Icons.Cpu,
  LogoOpenAI: Icons.Box,
  LogoOpenRouter: Icons.Globe,
  Brain: Icons.Brain,
  Wrench: Icons.Wrench,
  Eye: Icons.Eye,
};

export function ProviderIcon({ name, size = 20, className }: ProviderIconProps) {
  const Icon = ICON_MAP[name] || Icons.Bot;

  return <Icon className={className} style={{ width: size, height: size }} />;
}
