import { OpenTradesTable } from "@/components/open-trades-table";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Icons,
  Page,
  PageContent,
  PageHeader,
  Skeleton,
} from "@/ui";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { IncomeView } from "../../income/income-view";
import { AdaptiveCalendarView } from "../components/adaptive-calendar-view";
import { DistributionCharts } from "../components/distribution-charts";
import { EquityCurveChart } from "../components/equity-curve-chart";
import { KPISummaryCards } from "../components/kpi-summary-cards";
import { PeriodSelector, getChartPeriodDisplay } from "../components/period-selector";
import { SettingsSheet } from "../components/settings-sheet";
import { useSwingDashboard } from "../hooks/use-swing-dashboard";
import { useSwingPreferences } from "../hooks/use-swing-preferences";

export default function DashboardPage() {
  const { t } = useTranslation("trading");
  const navigate = useNavigate();
  const [selectedPeriod, setSelectedPeriod] = useState<"1M" | "3M" | "6M" | "YTD" | "1Y" | "ALL">(
    "YTD",
  );
  const [selectedYear, setSelectedYear] = useState(new Date());
  const [settingsOpen, setSettingsOpen] = useState(false);

  const { data: dashboardData, isLoading, error, refetch } = useSwingDashboard(selectedPeriod);
  const { preferences } = useSwingPreferences();

  const handleNavigateToActivities = () => {
    navigate("/trading/activities");
  };

  const renderDashboardContent = () => {
    if (isLoading) {
       return <div className="space-y-4 pt-4"><Skeleton className="h-[400px] w-full" /></div>;
    }

    if (error || !dashboardData) {
       return (
         <div className="flex h-[400px] items-center justify-center border rounded-lg bg-muted/10 mt-4">
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
       );
    }

    const { metrics, openPositions = [], periodPL = [], distribution, calendar = [] } = dashboardData;

    const hasSelectedActivities =
        preferences.selectedActivityIds.length > 0 || preferences.includeAllActivities;

    if (!hasSelectedActivities) {
        return (
            <div className="flex h-[400px] items-center justify-center border rounded-lg bg-muted/10 mt-4">
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

    return (
        <div className="space-y-4 sm:space-y-6 pt-4">
            <KPISummaryCards metrics={metrics} t={t} />

            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <PeriodSelector
                  selectedPeriod={selectedPeriod}
                  onPeriodSelect={setSelectedPeriod}
                  t={t}
                />
                <span className="text-muted-foreground border-l pl-2 ml-2 hidden text-xs sm:inline-block">
                  {getChartPeriodDisplay(selectedPeriod, t).description}
                </span>
              </div>

              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  className="hidden h-8 rounded-full text-xs sm:inline-flex"
                  onClick={handleNavigateToActivities}
                >
                  <Icons.ListChecks className="mr-2 h-3.5 w-3.5" />
                  <span>{t("dashboard.selectActivities")}</span>
                </Button>
                <Button
                  variant="outline"
                  size="icon"
                  onClick={handleNavigateToActivities}
                  className="h-8 w-8 sm:hidden"
                  aria-label="Select activities"
                >
                  <Icons.ListChecks className="h-3.5 w-3.5" />
                </Button>
              </div>
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
    );
  };

  const headerActions = (
    <div className="flex items-center gap-2">
      <TabsList className="bg-muted/50 p-1">
        <TabsTrigger value="dashboard" className="px-4 text-xs font-medium">{t("dashboard.tabs.dashboard")}</TabsTrigger>
        <TabsTrigger value="income" className="px-4 text-xs font-medium">{t("dashboard.tabs.income")}</TabsTrigger>
      </TabsList>

      <div className="h-6 w-px bg-border mx-2" />

       <Button
        variant="outline"
        size="icon"
        onClick={() => setSettingsOpen(true)}
        className="rounded-full bg-background"
      >
        <Icons.Settings className="size-4" />
      </Button>
    </div>
  );

  return (
    <Page>
      <Tabs defaultValue="dashboard" className="flex flex-col flex-1 h-full space-y-0">
        <PageHeader
            heading={t("dashboard.heading")}
            actions={headerActions}
        />

        <PageContent className="flex-1 space-y-4 pt-4">
            <TabsContent value="dashboard" className="space-y-4 m-0 h-full flex flex-col">
                {renderDashboardContent()}
            </TabsContent>
            <TabsContent value="income" className="space-y-4 m-0">
                 <IncomeView />
            </TabsContent>
        </PageContent>
      </Tabs>

      <SettingsSheet open={settingsOpen} onOpenChange={setSettingsOpen} />
    </Page>
  );
}
