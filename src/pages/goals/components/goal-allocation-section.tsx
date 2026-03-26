/**
 * Goal Allocation Section Component
 * Displays allocation settings and allocation history table
 */

import type { Account, Goal, GoalAllocation } from "@/lib/types";
import { Button, Icons } from "@/ui";
import { useTranslation } from "react-i18next";
import { AllocationHistoryTable } from "./allocation-history-table";
import GoalsAllocations from "./goal-allocations";

interface GoalAllocationSectionProps {
  goal: Goal;
  goals: Goal[];
  accounts: Account[];
  allocations: GoalAllocation[];
  currentAccountValues: Map<string, number>;
  getAllocationValue: (allocationId: string) => number | undefined;
  onEditAllocations: () => void;
  onAllocationUpdated: (allocation: GoalAllocation) => Promise<void>;
  onAllocationDeleted: (allocationId: string) => Promise<void>;
}

export function GoalAllocationSection({
  goal,
  goals,
  accounts,
  allocations,
  currentAccountValues,
  getAllocationValue,
  onEditAllocations,
  onAllocationUpdated,
  onAllocationDeleted,
}: GoalAllocationSectionProps) {
  const { t } = useTranslation("goals");
  const goalAllocations = allocations.filter((a) => a.goalId === goal.id);

  return (
    <div className="mb-8 space-y-8">
      {/* Allocation Settings - Hidden for completed goals */}
      {!goal.isAchieved && (
        <div>
          <div className="mb-4 flex items-start justify-between">
            <div>
              <h3 className="text-foreground mb-2 text-xl font-bold">
                {t("details.allocationSettings.title")}
              </h3>
              <p className="text-muted-foreground text-sm">
                {t("details.allocationSettings.description")}
              </p>
            </div>
            <Button onClick={onEditAllocations} variant="default">
              <Icons.Pencil className="mr-2 h-4 w-4" />
              {t("details.allocationSettings.editButton")}
            </Button>
          </div>

          {/* Allocation overview table */}
          <GoalsAllocations
            goals={[goal]}
            accounts={accounts}
            existingAllocations={goalAllocations}
            allAllocations={allocations}
            onSubmit={async () => {}}
            readOnly={true}
            showRemaining={true}
            currentAccountValues={currentAccountValues}
          />
        </div>
      )}

      {/* Current Allocations / Allocation History */}
      <div>
        <h3 className="text-foreground mb-2 text-xl font-bold">
          {goal.isAchieved ? t("completedGoal.allocationHistory") : t("details.allocations.title")}
        </h3>
        <p className="text-muted-foreground mb-4 text-sm">
          {goal.isAchieved
            ? t("completedGoal.historyDescription")
            : t("details.allocations.description")}
        </p>

        {/* Info message for completed goals */}
        {goal.isAchieved && (
          <div className="mb-4 flex items-center gap-2 rounded-lg border border-green-200 bg-green-50 px-4 py-3 text-sm text-green-700 dark:border-green-800 dark:bg-green-900/20 dark:text-green-400">
            <Icons.InfoCircle className="h-4 w-4 shrink-0" />
            <span>{t("completedGoal.allocationsReleased")}</span>
          </div>
        )}

        <AllocationHistoryTable
          goalId={goal.id}
          goalStartDate={goal.startDate}
          goalDueDate={goal.dueDate}
          allocations={goalAllocations}
          allAllocations={allocations}
          allGoals={goals}
          accounts={new Map(accounts.map((acc) => [acc.id, acc]))}
          currentAccountValues={currentAccountValues}
          getAllocationValue={getAllocationValue}
          onAllocationUpdated={onAllocationUpdated}
          onAllocationDeleted={onAllocationDeleted}
          readOnly={goal.isAchieved}
        />
      </div>
    </div>
  );
}
