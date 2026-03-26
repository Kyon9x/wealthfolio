import { Separator } from "@/components/ui/separator";
import { useTranslation } from "react-i18next";
import { SettingsHeader } from "../settings-header";
import { AiProviderSettings } from "./ai-provider-settings";

export default function AiProviderSettingsPage() {
  const { t } = useTranslation("settings");

  return (
    <div className="space-y-6">
      <SettingsHeader heading={t("ai.title")} text={t("ai.description")} />
      <Separator />
      <AiProviderSettings />
    </div>
  );
}
