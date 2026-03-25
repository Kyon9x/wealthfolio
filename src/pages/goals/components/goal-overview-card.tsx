/**
 * Goal Overview Card Component
 * Displays target amount, current progress, description, and key metrics
 */

import { MetricDisplay } from "@/components/metric-display";
import { formatTimeRemaining } from "@/lib/date-utils";
import type { Goal } from "@/lib/types";
import { formatAmount, Icons } from "@/ui";
import { useTranslation } from "react-i18next";

interface GoalOverviewCardProps {
  goal: Goal;
  currentAmount: number;
  progress: number;
  projectedFutureValue: number;
  onTrack: boolean;
}

export function GoalOverviewCard({
  goal,
  currentAmount,
  progress,
  projectedFutureValue,
  onTrack,
}: GoalOverviewCardProps) {
  const { t } = useTranslation("goals");
  const actualColor = onTrack ? "var(--chart-actual-on-track)" : "var(--chart-actual-off-track)";

  return (
    <div className="border-border bg-card col-span-1 flex flex-col rounded-xl border shadow-sm">
      <div className="border-border border-b px-6 py-4">
        <h3 className="text-foreground text-lg font-bold">{t("details.overview.title")}</h3>
      </div>
      <div className="flex flex-1 flex-col space-y-6 p-6">
        {/* Target Amount & Progress */}
        <div className="border-border flex items-start justify-between gap-4 border-b pb-6">
          <div className="flex-1 space-y-3">
            <div>
              <p className="text-muted-foreground mb-1 text-xs">
                {t("details.overview.targetAmount")}
              </p>
              <p className="text-2xl font-bold" style={{ color: "var(--chart-projected)" }}>
                {formatAmount(goal.targetAmount, "USD", false)}
              </p>
            </div>
            <div className="space-y-1.5">
              <p className="text-muted-foreground text-xs">
                {t("details.overview.currentProgress")}
              </p>
              <p className="text-sm font-medium">
                <span className="font-bold" style={{ color: actualColor }}>
                  {progress.toFixed(1)}%
                </span>{" "}
                -{" "}
                <span style={{ color: actualColor }}>
                  {formatAmount(currentAmount, "USD", false)}
                </span>
              </p>
            </div>
          </div>
          <div className="bg-primary/10 flex h-14 w-14 flex-shrink-0 items-center justify-center rounded-full">
            <Icons.Goal className="text-primary h-6 w-6" />
          </div>
        </div>

        {/* Description - if exists */}
        {goal.description && (
          <div className="border-border border-b pb-6">
            <p className="text-muted-foreground mb-2 text-xs">
              {t("details.overview.description")}
            </p>
            <div
              style={{
                overflow: "hidden",
                display: "-webkit-box",
                WebkitLineClamp: 2,
                WebkitBoxOrient: "vertical",
              }}
              className="text-foreground text-sm whitespace-pre-wrap"
            >
              {goal.description}
            </div>
          </div>
        )}

        {/* Metrics Grid - 2x2 */}
        <div className="grid grid-cols-2 gap-3">
          <MetricDisplay
            label={t("details.metrics.monthlyInvestmentDCA")}
            value={undefined}
            className="border-muted/30 bg-muted/30 rounded-md border"
            labelComponent={
              <div className="text-muted-foreground flex w-full flex-col items-center justify-center space-y-2 text-xs">
                <span className="text-center">{t("details.metrics.monthlyInvestment")}</span>
                <span className="text-foreground text-sm font-bold">
                  {goal.monthlyInvestment
                    ? formatAmount(goal.monthlyInvestment, "USD", false)
                    : t("details.metrics.notSet")}
                </span>
              </div>
            }
          />
          <MetricDisplay
            label={t("details.metrics.targetReturnRate")}
            value={goal.targetReturnRate ? goal.targetReturnRate / 100 : 0}
            isPercentage={true}
            className="border-muted/30 bg-muted/30 rounded-md border"
          />
          <MetricDisplay
            label={t("details.metrics.timeRemaining")}
            value={undefined}
            className="border-muted/30 bg-muted/30 rounded-md border"
            labelComponent={
              <div className="text-muted-foreground flex w-full flex-col items-center justify-center space-y-2 text-xs">
                <span className="text-center">{t("details.metrics.timeRemaining")}</span>
                <span className="text-foreground text-sm font-bold">
                  {formatTimeRemaining(goal.dueDate, t)}
                </span>
              </div>
            }
          />
          <MetricDisplay
            label={t("details.metrics.projectedFutureValue")}
            value={undefined}
            className="border-muted/30 bg-muted/30 rounded-md border"
            labelComponent={
              <div className="text-muted-foreground flex w-full flex-col items-center justify-center space-y-2 text-xs">
                <span className="text-center">{t("details.metrics.projectedFutureValue")}</span>
                <span className="text-sm font-bold" style={{ color: "var(--chart-projected)" }}>
                  {formatAmount(projectedFutureValue, "USD", false)}
                </span>
              </div>
            }
          />
        </div>
      </div>
    </div>
  );
}
