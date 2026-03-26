/**
 * Goal Chart Card Component
 * Displays the growth projection chart with time period selector
 */

import { AnimatedToggleGroup, formatAmount, Icons, Skeleton } from "@/ui";
import { useTranslation } from "react-i18next";
import {
    Area,
    AreaChart,
    CartesianGrid,
    Legend,
    ResponsiveContainer,
    Tooltip,
    XAxis,
    YAxis,
} from "recharts";
import type { GoalChartDataPoint, TimePeriodOption } from "../lib/goal-types";

interface GoalChartCardProps {
  chartData: GoalChartDataPoint[];
  isLoading: boolean;
  timePeriod: TimePeriodOption;
  onTimePeriodChange: (period: TimePeriodOption) => void;
  onTrack: boolean;
}

export function GoalChartCard({
  chartData,
  isLoading,
  timePeriod,
  onTimePeriodChange,
  onTrack,
}: GoalChartCardProps) {
  const { t } = useTranslation("goals");
  const actualColor = onTrack ? "var(--chart-actual-on-track)" : "var(--chart-actual-off-track)";

  // Time period options
  const timePeriodOptions = [
    { value: "weeks" as const, label: t("details.chart.periods.weeks") },
    { value: "months" as const, label: t("details.chart.periods.months") },
    { value: "years" as const, label: t("details.chart.periods.years") },
    { value: "all" as const, label: t("details.chart.periods.all") },
  ];

  // Format tooltip value
  const formatTooltipValue = (value: number | null) => {
    if (value === null || value === undefined) return "N/A";
    return formatAmount(value, "USD", false);
  };

  return (
    <div className="border-border bg-card col-span-1 rounded-xl border shadow-sm md:col-span-2">
      <div className="border-border flex items-center justify-between border-b px-6 py-4">
        <h3 className="text-foreground text-lg font-bold">
          {t("details.chart.growthProjection")}
        </h3>
        <AnimatedToggleGroup
          items={timePeriodOptions}
          value={timePeriod}
          onValueChange={(value) => onTimePeriodChange(value as TimePeriodOption)}
          variant="secondary"
          size="sm"
        />
      </div>
      <div className="p-0">
        <div className="h-[480px] w-full">
          {isLoading ? (
            <div className="flex h-full items-center justify-center">
              <Skeleton className="h-full w-full" />
            </div>
          ) : chartData.length === 0 ? (
            <div className="text-muted-foreground flex h-full flex-col items-center justify-center">
              <Icons.TrendingUp className="mb-2 h-12 w-12 opacity-50" />
              <p>{t("details.chart.noData")}</p>
              <p className="text-sm">{t("details.chart.noDataDescription")}</p>
            </div>
          ) : (
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chartData} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
                <defs>
                  <linearGradient id="colorProjected" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="var(--chart-projected)" stopOpacity={0.2} />
                    <stop offset="100%" stopColor="var(--chart-projected)" stopOpacity={0} />
                  </linearGradient>
                  <linearGradient id="colorActual" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor={actualColor} stopOpacity={0.2} />
                    <stop offset="100%" stopColor={actualColor} stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid
                  strokeDasharray="3 3"
                  vertical={false}
                  stroke="hsl(var(--border))"
                  strokeOpacity={0.5}
                />
                <XAxis
                  dataKey="dateLabel"
                  axisLine={false}
                  tickLine={false}
                  tick={{ fontSize: 11, fill: "currentColor" }}
                  minTickGap={20}
                />
                <YAxis
                  axisLine={false}
                  tickLine={false}
                  tick={{ fontSize: 11, fill: "currentColor" }}
                  tickFormatter={(value) => {
                    if (value >= 1000000000) return `${(value / 1000000000).toFixed(1)}B`;
                    if (value >= 1000000) return `${(value / 1000000).toFixed(1)}M`;
                    if (value >= 1000) return `${(value / 1000).toFixed(0)}K`;
                    return value.toString();
                  }}
                  width={60}
                />
                <Tooltip
                  cursor={{ stroke: "hsl(var(--border))", strokeWidth: 1 }}
                  contentStyle={{
                    borderRadius: "8px",
                    border: "1px solid hsl(var(--border))",
                    boxShadow: "0 4px 12px rgba(0, 0, 0, 0.15)",
                    backgroundColor: "hsl(var(--popover))",
                    color: "hsl(var(--popover-foreground))",
                    padding: "12px",
                  }}
                  formatter={(value, name) => [
                    formatTooltipValue(typeof value === "number" ? value : null),
                    name === "projected"
                      ? t("details.chart.projectedGrowth")
                      : t("details.chart.actualValue"),
                  ]}
                  labelFormatter={(label) => label}
                />
                <Legend
                  verticalAlign="top"
                  align="right"
                  height={36}
                  iconType="circle"
                  iconSize={8}
                  formatter={(value) =>
                    value === "projected"
                      ? t("details.chart.projectedGrowth")
                      : t("details.chart.actualValue")
                  }
                  wrapperStyle={{ fontSize: "12px" }}
                />
                <Area
                  type="monotone"
                  dataKey="projected"
                  stroke="var(--chart-projected)"
                  strokeWidth={2}
                  strokeDasharray="5 5"
                  fill="url(#colorProjected)"
                  name="projected"
                  connectNulls
                  dot={false}
                />
                <Area
                  type="monotone"
                  dataKey="actual"
                  stroke={actualColor}
                  strokeWidth={2}
                  fill="url(#colorActual)"
                  name="actual"
                  connectNulls
                  dot={false}
                  activeDot={{ r: 4, strokeWidth: 2 }}
                />
              </AreaChart>
            </ResponsiveContainer>
          )}
        </div>
      </div>
    </div>
  );
}
