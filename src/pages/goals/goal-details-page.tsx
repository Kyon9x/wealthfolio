/**
 * Goal Details Page
 * Main page component that orchestrates goal detail sub-components
 */

import { getGoals, getGoalsAllocation } from "@/commands/goal";
import { useAccounts } from "@/hooks/use-accounts";
import { useLatestValuations } from "@/hooks/use-latest-valuations";
import { QueryKeys } from "@/lib/query-keys";
import type { Goal, GoalAllocation } from "@/lib/types";
import { useQuery } from "@tanstack/react-query";
import { Button, Page, Skeleton } from "@/ui";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate, useParams } from "react-router-dom";

// Components
import { EditAllocationsModal } from "./components/edit-allocations-modal";
import { GoalAllocationSection } from "./components/goal-allocation-section";
import { GoalChartCard } from "./components/goal-chart-card";
import { GoalEditModal } from "./components/goal-edit-modal";
import { GoalHeader } from "./components/goal-header";
import { GoalOverviewCard } from "./components/goal-overview-card";

// Hooks
import { useGoalMutations } from "./hooks/use-goal-mutations";
import { useGoalProgress } from "./hooks/use-goal-progress";
import { useGoalValuationHistory } from "./hooks/use-goal-valuation-history";

// Utilities
import type { TimePeriodOption } from "./lib/goal-types";
import {
  calculateProjectedValueByDate,
  isGoalOnTrackByDate,
  parseGoalDate
} from "./lib/goal-utils";

export default function GoalDetailsPage() {
  const { t } = useTranslation("goals");
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  // State
  const [visibleModal, setVisibleModal] = useState(false);
  const [isCreatingAllocation, setIsCreatingAllocation] = useState(false);
  const [timePeriod, setTimePeriod] = useState<TimePeriodOption>("months");

  // Data fetching
  const { data: goals, isLoading: isGoalsLoading } = useQuery<Goal[], Error>({
    queryKey: [QueryKeys.GOALS],
    queryFn: getGoals,
  });

  const { data: allocations, isLoading: isAllocationsLoading } = useQuery<GoalAllocation[], Error>({
    queryKey: [QueryKeys.GOALS_ALLOCATIONS],
    queryFn: getGoalsAllocation,
  });

  const { accounts } = useAccounts();

  // Fetch latest valuations for accounts
  const accountIds = accounts?.map((acc) => acc.id) || [];
  const { latestValuations } = useLatestValuations(accountIds);

  // Build current account values map from valuations, fallback to account.balance
  const currentAccountValuesFromValuations = new Map(
    (accounts || []).map((acc) => {
      const valuation = (latestValuations || []).find((v) => v.accountId === acc.id);
      return [acc.id, valuation?.totalValue ?? acc.balance ?? 0];
    })
  );

  // Hooks
  const { getGoalProgress, getAllocationValue } = useGoalProgress(goals);
  const { updateAllocationMutation, deleteAllocationMutation, saveAllocationsMutation } =
    useGoalMutations();

  // Derived data
  const goal = goals?.find((g) => g.id === id);
  const goalProgress = id ? getGoalProgress(id) : undefined;

  // Get startValue from goalProgress for chart projection consistency
  const chartStartValue = goalProgress?.startValue ?? 0;

  // Calculate projectedFutureValue for chart hook
  const chartProjectedFutureValue = useMemo(() => {
    if (!goal || !goal.startDate || !goal.dueDate) return undefined;
    const startDate = parseGoalDate(goal.startDate);
    const dueDate = parseGoalDate(goal.dueDate);
    const annualReturnRate = goal.targetReturnRate ?? 0;

    return calculateProjectedValueByDate(
      goal.targetAmount,
      annualReturnRate,
      startDate,
      dueDate,
      dueDate
    );
  }, [goal, chartStartValue]);

  // Use the chart data hook
  const { chartData, isLoading: isChartLoading } = useGoalValuationHistory(goal, timePeriod, {
    startValue: chartStartValue,
    projectedFutureValue: chartProjectedFutureValue,
  });

  // Loading state
  if (isGoalsLoading || isAllocationsLoading) {
    return (
      <Page className="flex flex-col gap-6 p-4 md:p-6 lg:p-8">
        <Skeleton className="h-10 w-48" />
        <Skeleton className="h-64 w-full" />
      </Page>
    );
  }

  // Not found state
  if (!goal) {
    return (
      <Page className="flex h-screen flex-col items-center justify-center gap-4">
        <h1 className="text-2xl font-bold">{t("notFound")}</h1>
        <Button onClick={() => navigate("/goals")}>{t("backToGoals")}</Button>
      </Page>
    );
  }

  // Calculate progress values
  const currentAmount = goalProgress?.currentValue ?? 0;
  const progress =
    goal?.targetAmount && goal.targetAmount > 0
      ? Math.min((currentAmount / goal.targetAmount) * 100, 100)
      : 0;

  // Calculate projections
  let projectedFutureValue = 0;
  let onTrack = true;

  if (goal && goal.startDate && goal.dueDate && goalProgress) {
    const startDate = parseGoalDate(goal.startDate);
    const dueDate = parseGoalDate(goal.dueDate);
    const annualReturnRate = goal.targetReturnRate ?? 0;

    projectedFutureValue = calculateProjectedValueByDate(
      goal.targetAmount,
      annualReturnRate,
      startDate,
      dueDate,
      dueDate
    );

    onTrack = isGoalOnTrackByDate(
      currentAmount,
      goal.targetAmount,
      annualReturnRate,
      startDate,
      dueDate
    );
  }

  // For weeks/months views: Use the last chart point's projected value
  if ((timePeriod === "weeks" || timePeriod === "months") && chartData && chartData.length > 0) {
    const lastChartPoint = chartData[chartData.length - 1];
    if (lastChartPoint.projected !== null) {
      projectedFutureValue = lastChartPoint.projected;
    }
  }

  return (
    <Page className="flex flex-col gap-6 p-4 md:p-6 lg:p-8">
      {/* Header */}
      <GoalHeader
        goal={goal}
        onEdit={() => setVisibleModal(true)}
        onBack={() => navigate("/goals")}
      />

      {/* Chart & Stats Grid */}
      <div className="grid grid-cols-1 gap-4 pt-0 md:grid-cols-3">
        {/* Chart Card - 2 columns on desktop */}
        <GoalChartCard
          chartData={chartData}
          isLoading={isChartLoading}
          timePeriod={timePeriod}
          onTimePeriodChange={setTimePeriod}
          onTrack={onTrack}
        />

        {/* Overview Card - 1 column on desktop */}
        <GoalOverviewCard
          goal={goal}
          currentAmount={currentAmount}
          progress={progress}
          projectedFutureValue={projectedFutureValue}
          onTrack={onTrack}
        />
      </div>

      {/* Allocations Section */}
      <GoalAllocationSection
        goal={goal}
        goals={goals || []}
        accounts={accounts || []}
        allocations={allocations || []}
        currentAccountValues={currentAccountValuesFromValuations}
        getAllocationValue={getAllocationValue}
        onEditAllocations={() => setIsCreatingAllocation(true)}
        onAllocationUpdated={async (allocation) => {
          await updateAllocationMutation.mutateAsync(allocation);
        }}
        onAllocationDeleted={async (allocationId) => {
          await deleteAllocationMutation.mutateAsync(allocationId);
        }}
      />

      {/* Modals */}
      <GoalEditModal goal={goal} open={visibleModal} onClose={() => setVisibleModal(false)} />

      {goal && accounts && (
        <EditAllocationsModal
          open={isCreatingAllocation}
          onOpenChange={setIsCreatingAllocation}
          goal={goal}
          accounts={accounts}
          currentAccountValues={currentAccountValuesFromValuations}
          existingAllocations={allocations?.filter((a) => a.goalId === id) || []}
          allAllocations={allocations || []}
          allGoals={goals || []}
          onSubmit={async (newAllocations) => {
            await saveAllocationsMutation.mutateAsync(newAllocations);
          }}
        />
      )}
    </Page>
  );
}
