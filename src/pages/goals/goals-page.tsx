import { getGoals, getGoalsAllocation } from "@/commands/goal";
import { useAccounts } from "@/hooks/use-accounts";
import { useLatestValuations } from "@/hooks/use-latest-valuations";
import { QueryKeys } from "@/lib/query-keys";
import { useSettingsContext } from "@/lib/settings-provider";
import type { Goal, GoalAllocation } from "@/lib/types";
import { useQuery } from "@tanstack/react-query";
import { Button, EmptyPlaceholder, Icons, Page, Skeleton } from "@/ui";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import GoalsAllocations from "./components/goal-allocations";
import { GoalEditModal } from "./components/goal-edit-modal";
import { GoalItem } from "./components/goal-item";
import { useGoalMutations } from "./hooks/use-goal-mutations";

const GoalsPage = () => {
  const { t } = useTranslation("goals");
  const { data: goals, isLoading } = useQuery<Goal[], Error>({
    queryKey: [QueryKeys.GOALS],
    queryFn: getGoals,
  });

  const { data: allocations } = useQuery<GoalAllocation[], Error>({
    queryKey: [QueryKeys.GOALS_ALLOCATIONS],
    queryFn: getGoalsAllocation,
  });

  const { accounts } = useAccounts();
  const { settings } = useSettingsContext();

  // Fetch latest valuations for accounts
  const accountIds = accounts?.map(acc => acc.id) || [];
  const { latestValuations } = useLatestValuations(accountIds);

  // Build current account values map from valuations (not from account.balance which may be stale)
  const currentAccountValuesFromValuations = new Map(
    (latestValuations || []).map(val => [val.accountId, val.totalValue])
  );

  const [visibleModal, setVisibleModal] = useState(false);
  const [selectedGoal, setSelectedGoal] = useState<Goal | null>(null);

  const { deleteGoalMutation, saveAllocationsMutation, updateGoalMutation } = useGoalMutations();

  const handleAddGoal = () => {
    setSelectedGoal(null);
    setVisibleModal(true);
  };

  const handleEditGoal = (goal: Goal) => {
    setSelectedGoal(goal);
    setVisibleModal(true);
  };

  const handleDeleteGoal = (goal: Goal) => {
    deleteGoalMutation.mutate(goal.id);
  };

  const handleCompleteGoal = (goal: Goal) => {
    updateGoalMutation.mutate({
      ...goal,
      isAchieved: true,
    });
  };

  const handleAddAllocation = (allocationData: GoalAllocation[]) => {
    saveAllocationsMutation.mutate(allocationData);
  };

  if (isLoading) {
    return (
      <Page className="flex flex-col gap-6 p-4 md:p-6 lg:p-8">
        <div className="flex items-center justify-between">
          <Skeleton className="h-10 w-48" />
          <Skeleton className="h-10 w-32" />
        </div>
        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
          <GoalItem.Skeleton />
          <GoalItem.Skeleton />
          <GoalItem.Skeleton />
        </div>
      </Page>
    );
  }

  return (
    <Page className="flex flex-col gap-8 p-4 md:p-6 lg:p-8">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">{t("title")}</h1>
          <p className="text-muted-foreground mt-2">{t("description")}</p>
        </div>
        <div className="flex items-center gap-2">
          <Button className="hidden sm:inline-flex" onClick={() => handleAddGoal()}>
            <Icons.Plus className="mr-2 h-4 w-4" />
            {t("addButton")}
          </Button>
          <Button
            size="icon"
            className="sm:hidden"
            onClick={() => handleAddGoal()}
            aria-label={t("addButton")}
          >
            <Icons.Plus className="h-4 w-4" />
          </Button>
        </div>
      </div>

      <div className="w-full space-y-8">
        {goals?.length ? (
          <>
            <div>
              <h2 className="text-foreground mb-6 text-xl font-bold">{t("goalsHeading")}</h2>
              <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
              {goals.map((goal: Goal) => {
                  const goalAllocations = allocations?.filter((a) => a.goalId === goal.id) ?? [];

                  return (
                    <GoalItem
                      key={goal.id}
                      goal={goal}
                      goals={goals}
                      allocations={goalAllocations}
                      totalAccountCount={accounts?.length ?? 0}
                      onEdit={handleEditGoal}
                      onDelete={handleDeleteGoal}
                      onComplete={handleCompleteGoal}
                    />
                  );
                })}
              </div>
            </div>

            <div>
              <div className="mb-6 flex items-center justify-between">
                <div>
                  <h2 className="text-foreground text-xl font-bold">{t("allocationsHeading")}</h2>
                  <p className="text-muted-foreground text-sm">{t("allocationsDescription")}</p>
                </div>
               </div>
              <GoalsAllocations
                 goals={goals}
                 existingAllocations={allocations || []}
                 accounts={accounts || []}
                 onSubmit={handleAddAllocation}
                 readOnly={true}
                 currentAccountValues={currentAccountValuesFromValuations}
                 currency={settings?.baseCurrency || "USD"}
               />
            </div>
          </>
        ) : (
          <EmptyPlaceholder>
            <EmptyPlaceholder.Icon name="Goal" />
            <EmptyPlaceholder.Title>{t("empty.title")}</EmptyPlaceholder.Title>
            <EmptyPlaceholder.Description>{t("empty.description")}</EmptyPlaceholder.Description>
            <Button onClick={() => handleAddGoal()}>
              <Icons.Plus className="mr-2 h-4 w-4" />
              {t("addGoalButton")}
            </Button>
          </EmptyPlaceholder>
        )}
      </div>

      <GoalEditModal
        goal={selectedGoal || undefined}
        open={visibleModal}
        onClose={() => setVisibleModal(false)}
      />
    </Page>
  );
};

export default GoalsPage;
