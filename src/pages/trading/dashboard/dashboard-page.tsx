import { AdaptiveCalendarView } from "../components/adaptive-calendar-view";
import { DistributionCharts } from "../components/distribution-charts";
import { EquityCurveChart } from "../components/equity-curve-chart";
import { OpenTradesTable } from "../components/open-trades-table";
import { useSwingDashboard } from "../hooks/use-swing-dashboard";
import { useSwingPreferences } from "../hooks/use-swing-preferences";
import {
  AnimatedToggleGroup,
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  GainAmount,
  GainPercent,
  Icons,
  Page,
  PageContent,
  PageHeader,
  Skeleton,
} from "@wealthfolio/ui";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";

// Chart period type is now automatically determined based on selected period
const getChartPeriodDisplay = (
  period: "1M" | "3M" | "6M" | "YTD" | "1Y" | "ALL",
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  t: any,
) => {
  switch (period) {
    case "1M":
      return {
        type: t("dashboard.chartPeriod.daily"),
        description: t("dashboard.chartPeriod.dailyDescription"),
      };
    case "3M":
      return {
        type: t("dashboard.chartPeriod.weekly"),
        description: t("dashboard.chartPeriod.weeklyDescription"),
      };
    default:
      return {
        type: t("dashboard.chartPeriod.monthly"),
        description: t("dashboard.chartPeriod.monthlyDescription"),
      };
  }
};

const PeriodSelector: React.FC<{
  selectedPeriod: "1M" | "3M" | "6M" | "YTD" | "1Y" | "ALL";
  onPeriodSelect: (period: "1M" | "3M" | "6M" | "YTD" | "1Y" | "ALL") => void;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  t: any;
}> = ({ selectedPeriod, onPeriodSelect, t }) => {
  const periods = [
    { value: "1M" as const, label: t("dashboard.periods.1M") },
    { value: "3M" as const, label: t("dashboard.periods.3M") },
    { value: "6M" as const, label: t("dashboard.periods.6M") },
    { value: "YTD" as const, label: t("dashboard.periods.YTD") },
    { value: "1Y" as const, label: t("dashboard.periods.1Y") },
    { value: "ALL" as const, label: t("dashboard.periods.ALL") },
  ];

  return (
    <AnimatedToggleGroup
      items={periods}
      value={selectedPeriod}
      onValueChange={onPeriodSelect}
      variant="secondary"
      size="sm"
    />
  );
};

export default function DashboardPage() {
  const { t } = useTranslation("trading");
  const navigate = useNavigate();
  const [selectedPeriod, setSelectedPeriod] = useState<"1M" | "3M" | "6M" | "YTD" | "1Y" | "ALL">(
    "YTD",
  );
  const [selectedYear, setSelectedYear] = useState(new Date());

  const { data: dashboardData, isLoading, error, refetch } = useSwingDashboard(selectedPeriod);
  const { preferences } = useSwingPreferences();

  const handleNavigateToActivities = () => {
    navigate("/trading/activities");
  };

  const handleNavigateToSettings = () => {
    navigate("/trading/settings");
  };

  if (isLoading) {
    return <DashboardSkeleton />;
  }

  if (error || !dashboardData) {
    return (
      <Page>
        <PageHeader heading={t("dashboard.heading")} />
        <PageContent>
          <div className="flex h-[calc(100vh-200px)] items-center justify-center">
            <div className="px-4 text-center">
              <Icons.AlertCircle className="text-muted-foreground mx-auto mb-4 h-10 w-10 sm:h-12 sm:w-12" />
              <h3 className="mb-2 text-base font-semibold sm:text-lg">
                {t("dashboard.error.heading")}
              </h3>
              <p className="text-muted-foreground mb-4 text-sm sm:text-base">
                {error?.message || t("dashboard.error.message")}
              </p>
              <Button onClick={() => refetch()}>{t("dashboard.tryAgain")}</Button>
            </div>
          </div>
        </PageContent>
      </Page>
    );
  }

  const { metrics, openPositions = [], periodPL = [], distribution, calendar = [] } = dashboardData;

  const hasSelectedActivities =
    preferences.selectedActivityIds.length > 0 || preferences.includeSwingTag;
  if (!hasSelectedActivities) {
    return (
      <Page>
        <PageHeader heading={t("dashboard.heading")} />
        <PageContent>
          <div className="flex h-[calc(100vh-200px)] items-center justify-center">
            <div className="px-4 text-center">
              <Icons.BarChart className="text-muted-foreground mx-auto mb-4 h-10 w-10 sm:h-12 sm:w-12" />
              <h3 className="mb-2 text-base font-semibold sm:text-lg">
                {t("dashboard.emptyState.heading")}
              </h3>
              <p className="text-muted-foreground mb-4 text-sm sm:text-base">
                {t("dashboard.emptyState.message")}
              </p>
              <Button onClick={handleNavigateToActivities} className="mx-auto">
                <Icons.Plus className="mr-2 h-4 w-4" />
                {t("dashboard.emptyState.button")}
              </Button>
            </div>
          </div>
        </PageContent>
      </Page>
    );
  }

  // Transform PeriodPL data to EquityPoint format for chart
  const chartEquityData = periodPL.map((period, index) => {
    // Calculate cumulative P/L up to this period
    const cumulativeRealizedPL = periodPL
      .slice(0, index + 1)
      .reduce((sum, p) => sum + p.realizedPL, 0);

    return {
      date: period.date,
      cumulativeRealizedPL,
      cumulativeTotalPL: cumulativeRealizedPL, // For now, same as realized
      currency: period.currency,
    };
  });

  const headerActions = (
    <>
      <PeriodSelector selectedPeriod={selectedPeriod} onPeriodSelect={setSelectedPeriod} t={t} />
      <Button
        variant="outline"
        className="hidden rounded-full sm:inline-flex"
        onClick={handleNavigateToActivities}
      >
        <Icons.ListChecks className="mr-2 h-4 w-4" />
        <span>{t("dashboard.selectActivities")}</span>
      </Button>
      <Button
        variant="outline"
        size="icon"
        onClick={handleNavigateToActivities}
        className="sm:hidden"
        aria-label="Select activities"
      >
        <Icons.ListChecks className="h-4 w-4" />
      </Button>

      <Button
        variant="outline"
        size="icon"
        onClick={handleNavigateToSettings}
        className="rounded-full"
      >
        <Icons.Settings className="size-4" />
      </Button>
    </>
  );

  return (
    <Page>
      <PageHeader heading={t("dashboard.heading")} actions={headerActions} />

      <PageContent>
        <div className="space-y-4 sm:space-y-6">
          {/* KPI Cards */}
          <div className="grid grid-cols-1 gap-3 sm:gap-4 md:grid-cols-3">
            {/* Widget 1: Overall P/L Summary - Clean Design */}

            <Card
              className={`${metrics.totalPL >= 0 ? "border-success/10 bg-success/10" : "border-destructive/10 bg-destructive/10"}`}
            >
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pt-4 pb-3">
                <CardTitle className="text-sm font-medium">{t("dashboard.kpi.pl.title")}</CardTitle>
                <GainAmount
                  className="text-xl font-bold sm:text-2xl"
                  value={metrics.totalPL}
                  currency={metrics.currency}
                />
              </CardHeader>
              <CardContent className="space-y-3">
                {/* Details Below - Labels Left, Amounts Right */}
                <div className="space-y-2 pt-2">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground text-xs">
                      {t("dashboard.kpi.pl.realized", { count: metrics.totalTrades })}
                    </span>
                    <div className="flex items-center gap-2">
                      <GainAmount
                        value={metrics.totalRealizedPL}
                        currency={metrics.currency}
                        className="font-medium"
                        displayDecimal={false}
                      />
                    </div>
                  </div>
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground text-xs">
                      {t("dashboard.kpi.pl.unrealized", { count: metrics.openPositions })}
                    </span>
                    <div className="flex items-center gap-2">
                      <GainAmount
                        value={metrics.totalUnrealizedPL}
                        currency={metrics.currency}
                        className="font-medium"
                        displayDecimal={false}
                      />
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Widget 2: Core Performance */}
            <Card className="border-blue-500/10 bg-blue-500/10">
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">
                  {t("dashboard.kpi.corePerformance.title")}
                </CardTitle>
                <Icons.CheckCircle className="text-muted-foreground h-4 w-4" />
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-xs">
                      {t("dashboard.kpi.corePerformance.winRate")}
                    </span>
                    <GainPercent
                      value={metrics.winRate}
                      className="text-sm font-semibold"
                      showSign={false}
                    />
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-xs">
                      {t("dashboard.kpi.corePerformance.avgWin")}
                    </span>
                    <GainAmount
                      value={metrics.averageWin}
                      currency={metrics.currency}
                      className="text-sm font-semibold"
                      displayDecimal={false}
                    />
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-xs">
                      {t("dashboard.kpi.corePerformance.avgLoss")}
                    </span>
                    <GainAmount
                      value={-metrics.averageLoss}
                      currency={metrics.currency}
                      className="text-sm font-semibold"
                      displayDecimal={false}
                    />
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-xs">
                      {t("dashboard.kpi.corePerformance.totalTrades")}
                    </span>
                    <span className="text-sm font-semibold">{metrics.totalTrades}</span>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Widget 3: Analytics & Ratios */}
            <Card className="border-purple-500/10 bg-purple-500/10">
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">
                  {t("dashboard.kpi.analytics.title")}
                </CardTitle>
                <Icons.BarChart className="text-muted-foreground h-4 w-4" />
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-xs">
                      {t("dashboard.kpi.analytics.expectancy")}
                    </span>
                    <GainAmount
                      value={metrics.expectancy}
                      currency={metrics.currency}
                      className="text-sm font-semibold"
                      displayDecimal={false}
                    />
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-xs">
                      {t("dashboard.kpi.analytics.profitFactor")}
                    </span>
                    <span className="text-sm font-semibold">
                      {metrics.profitFactor === Number.POSITIVE_INFINITY
                        ? "∞"
                        : metrics.profitFactor.toFixed(2)}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-xs">
                      {t("dashboard.kpi.analytics.avgHoldTime")}
                    </span>
                    <span className="text-sm font-semibold">
                      {metrics.averageHoldingDays.toFixed(1)}{" "}
                      {t("dashboard.kpi.analytics.daysUnit")}
                    </span>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Charts Row - Equity Curve and Calendar */}
          <div className="grid grid-cols-1 gap-4 sm:gap-6 md:grid-cols-2">
            {/* Equity Curve */}
            <Card className="flex flex-col">
              <CardHeader className="shrink-0 pb-3">
                <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                  <div>
                    <CardTitle className="text-base sm:text-lg">
                      {t("dashboard.charts.equityCurve.title", {
                        period: getChartPeriodDisplay(selectedPeriod, t).type,
                      })}
                    </CardTitle>
                    <p className="text-muted-foreground text-xs sm:text-sm">
                      {getChartPeriodDisplay(selectedPeriod, t).description}
                    </p>
                  </div>
                  <div className="bg-secondary text-muted-foreground self-start rounded-full px-2 py-1 text-xs whitespace-nowrap sm:self-auto">
                    {t("dashboard.charts.equityCurve.periodDisplay", {
                      selectedPeriod: selectedPeriod,
                      periodType: getChartPeriodDisplay(selectedPeriod, t).type,
                    })}
                  </div>
                </div>
              </CardHeader>
              <CardContent className="flex min-h-0 flex-1 flex-col py-4 sm:py-6">
                <EquityCurveChart
                  data={chartEquityData}
                  currency={metrics.currency}
                  periodType={
                    selectedPeriod === "1M"
                      ? "daily"
                      : selectedPeriod === "3M"
                        ? "weekly"
                        : "monthly"
                  }
                />
              </CardContent>
            </Card>
            <Card className="flex flex-col pt-0">
              <CardContent className="flex min-h-0 flex-1 flex-col py-4 sm:py-6">
                <AdaptiveCalendarView
                  calendar={calendar}
                  selectedPeriod={selectedPeriod}
                  selectedYear={selectedYear}
                  onYearChange={setSelectedYear}
                  currency={metrics.currency}
                />
              </CardContent>
            </Card>
          </div>

          {/* Open Positions - Full Width on Mobile */}
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-base sm:text-lg">
                {t("dashboard.openPositions.title")}
              </CardTitle>
              <span className="text-muted-foreground text-sm">
                {openPositions.length}{" "}
                {openPositions.length === 1
                  ? t("dashboard.openPositions.position")
                  : t("dashboard.openPositions.positions")}
              </span>
            </CardHeader>
            <CardContent className="px-2 sm:px-6">
              <OpenTradesTable positions={openPositions} />
            </CardContent>
          </Card>

          {/* Distribution Charts */}
          <DistributionCharts distribution={distribution} currency={metrics.currency} />
        </div>
      </PageContent>
    </Page>
  );
}

function DashboardSkeleton() {
  return (
    <Page>
      <PageHeader
        heading="Trading Dashboard"
        text="Track your trading performance and analytics"
        actions={
          <>
            <Skeleton className="h-9 w-[280px]" />
            <Skeleton className="h-9 w-[100px] sm:w-[140px]" />
            <Skeleton className="h-9 w-9" />
          </>
        }
      />

      <PageContent>
        <div className="space-y-4 sm:space-y-6">
          <div className="grid grid-cols-1 gap-3 sm:gap-4 md:grid-cols-3">
            {[...Array(3)].map((_, index) => (
              <Card key={index}>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <Skeleton className="h-4 w-[100px] sm:w-[120px]" />
                  <Skeleton className="h-4 w-4" />
                </CardHeader>
                <CardContent>
                  <Skeleton className="h-6 w-[120px] sm:h-8 sm:w-[150px]" />
                  <Skeleton className="mt-2 h-3 w-[80px] sm:h-4 sm:w-[100px]" />
                </CardContent>
              </Card>
            ))}
          </div>

          <div className="grid grid-cols-1 gap-4 sm:gap-6 xl:grid-cols-2">
            <Card>
              <CardHeader>
                <Skeleton className="h-5 w-[120px] sm:h-6 sm:w-[150px]" />
              </CardHeader>
              <CardContent>
                <Skeleton className="h-[250px] w-full sm:h-[300px]" />
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <Skeleton className="h-5 w-[150px] sm:h-6 sm:w-[180px]" />
              </CardHeader>
              <CardContent>
                <div className="space-y-3 sm:space-y-4">
                  {[...Array(5)].map((_, index) => (
                    <div key={index} className="flex justify-between">
                      <Skeleton className="h-3 w-[80px] sm:h-4 sm:w-[100px]" />
                      <Skeleton className="h-3 w-[60px] sm:h-4 sm:w-[80px]" />
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </PageContent>
    </Page>
  );
}
