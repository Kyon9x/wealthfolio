import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { toast } from "@/components/ui/use-toast";
import { Account, Goal, GoalAllocation } from "@/lib/types";
import { formatAmount } from "@/ui";
import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

interface GoalsAllocationsProps {
  goals: Goal[];
  accounts: Account[];
  existingAllocations?: GoalAllocation[];
  allAllocations?: GoalAllocation[]; // All allocations including other goals
  onSubmit: (allocations: GoalAllocation[]) => void;
  readOnly?: boolean;
  showRemaining?: boolean;
  currentAccountValues?: Map<string, number>; // For displaying remaining value in dollars
  currency?: string; // User's base currency
}

const GoalsAllocations: React.FC<GoalsAllocationsProps> = ({
  goals,
  accounts,
  existingAllocations,
  allAllocations,
  onSubmit,
  readOnly = false,
  showRemaining = false,
  currentAccountValues = new Map(),
  currency = "USD",
}) => {
  const { t } = useTranslation("goals");
  const [allocations, setAllocations] = useState<GoalAllocation[]>(existingAllocations || []);
  const [totalAllocations, setTotalAllocations] = useState<Record<string, number>>({});
  const [isExceeding, setIsExceeding] = useState<boolean>(false);

  // Sync local state with props when existingAllocations changes
  useEffect(() => {
    if (existingAllocations) {
      setAllocations(existingAllocations);
    }
  }, [existingAllocations]);

  useEffect(() => {
    // When showing remaining, calculate from all allocations
    const sourceAllocations = showRemaining && allAllocations ? allAllocations : allocations;

    const totals = accounts.reduce(
      (acc, account) => {
        acc[account.id] = sourceAllocations.reduce((sum, alloc) => {
          if (alloc.accountId === account.id) {
            return sum + (alloc.allocatedPercent || 0);
          }
          return sum;
        }, 0);
        return acc;
      },
      {} as Record<string, number>,
    );

    setTotalAllocations(totals);
    setIsExceeding(Object.values(totals).some((total) => total > 100));
  }, [allocations, accounts, allAllocations, showRemaining]);

  const handleAllocationChange = (goalId: string, accountId: string, value: number) => {
    const updatedAllocations = allocations.map((alloc) =>
      alloc.goalId === goalId && alloc.accountId === accountId
        ? { ...alloc, allocatedPercent: value }
        : alloc,
    );
    if (
      !updatedAllocations.some((alloc) => alloc.goalId === goalId && alloc.accountId === accountId)
    ) {
      if (value > 0) {
        // Only add allocation if value is greater than 0 (avoid creating empty allocations)
        updatedAllocations.push({
          id: `${goalId}-${accountId}`,
          goalId,
          accountId,
          allocatedPercent: value,
          initialContribution: 0,
        } as GoalAllocation);
      }
    }
    setAllocations(updatedAllocations);
  };

  const handleSubmit = () => {
    if (isExceeding) {
      toast({
        title: t("allocations.toast.errorTitle"),
        className: "bg-red-500 text-white border-none",
      });
      return;
    }
    onSubmit(allocations);
  };

  return (
    <>
      <div className="relative overflow-x-auto rounded-md border">
        <table className="min-w-full table-auto">
          <thead>
            <tr>
              <th className="bg-muted sticky left-0 z-10 px-4 py-2 text-sm font-normal">
                {t("allocations.tableHeader")}
              </th>
              {accounts.map((account) => (
                <th key={account.id} className="border-l px-4 py-2 text-xs font-normal">
                  {account.name}
                </th>
              ))}
            </tr>
            {!showRemaining && (
              <>
                {/* Unallocated Row */}
                <tr>
                  <td className="bg-muted text-muted-foreground sticky left-0 z-10 border-t border-r px-4 py-2 text-xs">
                    {t("allocations.unallocated")}
                  </td>
                  {accounts.map((account) => {
                    const remainingPercent = Math.max(0, 100 - (totalAllocations[account.id] || 0));
                    const currentValue = currentAccountValues.get(account.id) || 0;
                    // Unallocated value = current account value × (unallocated percentage / 100)
                    // If 100% is allocated, unallocated value = 0
                    const remainingValue = currentValue * (remainingPercent / 100);
                    return (
                      <td
                        key={account.id}
                        className={`text-muted-foreground border-t border-l px-4 py-2 text-right text-xs ${
                          remainingPercent < 0 ? "text-destructive" : ""
                        }`}
                      >
                        <div className="flex flex-col items-end gap-1">
                          <span className={remainingPercent < 0 ? "text-destructive font-semibold" : "font-semibold"}>{remainingPercent}%</span>
                          <span className="text-xs">{formatAmount(remainingValue, currency, false)}</span>
                        </div>
                      </td>
                    );
                  })}
                </tr>
              </>
            )}
            {showRemaining && (
              <>
                {/* Unallocated Row - Combined Percentage and Value */}
                <tr>
                  <td className="bg-muted sticky left-0 z-10 border-t border-r p-0 text-xs font-semibold">
                    <div className="p-2">
                      <span>{t("allocations.unallocated")}</span>
                    </div>
                  </td>
                  {accounts.map((account) => {
                    const remainingPercent = Math.max(0, 100 - (totalAllocations[account.id] || 0));
                    const currentValue = currentAccountValues.get(account.id) || 0;
                    // Unallocated value = current account value × (unallocated percentage / 100)
                    // If 100% is allocated, unallocated value = 0
                    const remainingValue = currentValue * (remainingPercent / 100);
                    return (
                      <td key={account.id} className="text-muted-foreground border-t border-l px-4 py-2 text-right text-xs">
                        <div className="flex flex-col items-end gap-1">
                          <span className="font-semibold">{remainingPercent}%</span>
                          <span className="text-xs">{formatAmount(remainingValue, currency, false)}</span>
                        </div>
                      </td>
                    );
                  })}
                </tr>
              </>
            )}
                    </thead>
                    <tbody>
                    {goals.map((goal) => (
                    <React.Fragment key={goal.id}>
                    {/* Percentage Allocation Row */}
                    <tr className="border-t">
                    <td className="border-nones bg-muted sticky left-0 z-10 border-r p-0 text-xs font-semibold" rowSpan={2}>
                    <div className="p-2">
                    <span>{goal.title}</span>
                    <p className="text-muted-foreground text-xs font-light">
                      {formatAmount(goal.targetAmount, currency, false)}
                      </p>
                    </div>
                  </td>
                  {accounts.map((account) => {
                    const existingAllocation = allocations.find(
                      (alloc) => alloc.goalId === goal.id && alloc.accountId === account.id,
                    );
                    return (
                      <td key={account.id} className="border-r px-1 py-0">
                         <Input
                           className="m-0 h-full w-full rounded-none border-none px-2 text-right text-xs"
                           value={existingAllocation ? existingAllocation.allocatedPercent : ""}
                           onChange={(e) =>
                             handleAllocationChange(goal.id, account.id, Number(e.target.value))
                           }
                           disabled={readOnly}
                           placeholder="%"
                         />
                       </td>
                    );
                  })}
                </tr>
                {/* Initial Contribution Amount Row */}
                <tr className="border-t">
                  {accounts.map((account) => {
                    const existingAllocation = allocations.find(
                      (alloc) => alloc.goalId === goal.id && alloc.accountId === account.id,
                    );
                    return (
                      <td key={account.id} className="border-r px-1 py-0 text-muted-foreground text-xs">
                        <div className="px-2 py-1 text-right">
                          {existingAllocation ? formatAmount(existingAllocation.initialContribution || 0, currency, false) : "-"}
                        </div>
                      </td>
                    );
                  })}
                </tr>
              </React.Fragment>
            ))}
          </tbody>
        </table>
      </div>
      {!readOnly && (
        <div className="mt-4 text-right">
          <Button onClick={handleSubmit} disabled={isExceeding}>
            {t("allocations.saveButton")}
          </Button>
        </div>
      )}
    </>
  );
};

export default GoalsAllocations;
