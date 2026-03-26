import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import {
    Dialog,
    DialogContent,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { QueryKeys } from "@/lib/query-keys";
import { Account, Goal, GoalAllocation } from "@/lib/types";
import { cn } from "@/lib/utils";
import { useQueryClient } from "@tanstack/react-query";
import { formatAmount } from "@/ui";
import { Percent } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { calculateOtherGoalsPercentage, useAllocationForm } from "../hooks/use-allocation-form";

interface EditAllocationsModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  goal: Goal;
  accounts: Account[];
  currentAccountValues: Map<string, number>;
  existingAllocations?: GoalAllocation[]; // Current goal's allocations (for prefilling)
  allAllocations?: GoalAllocation[]; // All allocations for calculation
  allGoals?: Goal[]; // All goals for checking isAchieved status
  onSubmit: (allocations: GoalAllocation[]) => Promise<void>;
}

export function EditAllocationsModal({
  open,
  onOpenChange,
  goal,
  accounts,
  currentAccountValues,
  existingAllocations = [],
  allAllocations = [],
  allGoals = [],
  onSubmit,
}: EditAllocationsModalProps) {
  const { t } = useTranslation("goals");
  const queryClient = useQueryClient();
  const [allocations, setAllocations] = useState<Record<string, { allocationAmount: number; allocatedPercent: number }>>({});
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [isLoading, setIsLoading] = useState(false);

  // Use the shared allocation form hook
  const {
    availableBalances,
    isFetchingHistory,
    isGoalAchieved,
  } = useAllocationForm({
    goal,
    accounts,
    currentAccountValues,
    allAllocations,
    allGoals,
    open,
  });

  // Initial Data Refetch
  useEffect(() => {
    if (open) {
      queryClient.invalidateQueries({ queryKey: [QueryKeys.GOALS_ALLOCATIONS] });
      queryClient.refetchQueries({ queryKey: [QueryKeys.GOALS_ALLOCATIONS] });
    }
  }, [open, queryClient]);

  // Prefill form
  useEffect(() => {
    if (open) {
      const prefilledAllocations: Record<string, { allocationAmount: number; allocatedPercent: number }> = {};
      for (const account of accounts) {
        const existingAlloc = existingAllocations.find(
          (alloc) => alloc.accountId === account.id
        );

        if (existingAlloc) {
          prefilledAllocations[account.id] = {
            allocationAmount: existingAlloc.initialContribution,
            allocatedPercent: existingAlloc.allocatedPercent || 0,
          };
        }
      }
      setAllocations(prefilledAllocations);
      setErrors({}); // Clear errors on open/re-prefill
    }
  }, [open, accounts, existingAllocations]);

  // Handle modal open/close
  const handleOpenChange = (newOpen: boolean) => {
    onOpenChange(newOpen);
  };

  // Handle amount change - INDEPENDENT from percentage
  const handleAmountChange = (accountId: string, value: number) => {
    setAllocations((prev) => ({
      ...prev,
      [accountId]: {
        allocationAmount: value,
        allocatedPercent: prev[accountId]?.allocatedPercent || 0,
      },
    }));
    setErrors((prev) => ({ ...prev, [accountId]: "" }));
  };

  // Handle percentage change - INDEPENDENT from amount
  const handlePercentageChange = (accountId: string, value: number) => {
    setAllocations((prev) => ({
      ...prev,
      [accountId]: {
        allocationAmount: prev[accountId]?.allocationAmount || 0,
        allocatedPercent: value,
      },
    }));
    setErrors((prev) => ({ ...prev, [accountId]: "" }));
  };

  // Validate form
  const validateForm = async (): Promise<boolean> => {
    const newErrors: Record<string, string> = {};

    for (const account of accounts) {
      const alloc = allocations[account.id];

      // Skip if no allocation set for this account
      if (!alloc || (alloc.allocationAmount === 0 && alloc.allocatedPercent === 0)) continue;

      // Validate amount - must be > 0 if set
      if (alloc.allocationAmount < 0) {
        newErrors[account.id] = t("editAllocationsModal.errors.amountNegative");
        continue;
      }

      // Validate percentage - must be between 0 and 100
      if (alloc.allocatedPercent < 0 || alloc.allocatedPercent > 100) {
        newErrors[account.id] = "Percentage must be between 0 and 100";
        continue;
      }

      // Check available balance for amount
      const available = availableBalances[account.id] || 0;
      if (alloc.allocationAmount > available) {
        newErrors[account.id] = `Amount exceeds available balance (${formatAmount(available, account.currency, false)})`;
        continue;
      }

      // Check total percentage across all goals for this account
      const otherGoalsPercentage = calculateOtherGoalsPercentage(
        account.id,
        goal.id,
        goal.startDate,
        goal.dueDate,
        allAllocations,
        isGoalAchieved
      );

      const totalPercentage = otherGoalsPercentage + alloc.allocatedPercent;
      if (totalPercentage > 100) {
        newErrors[account.id] = `Total allocation exceeds 100% (other goals: ${otherGoalsPercentage.toFixed(1)}%, this: ${alloc.allocatedPercent.toFixed(1)}%)`;
        continue;
      }
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  // Handle submit
  const handleSubmit = async () => {
    if (!(await validateForm())) {
      return;
    }

    setIsLoading(true);
    try {
      const updatedAllocations: GoalAllocation[] = [];

      for (const account of accounts) {
         const alloc = allocations[account.id];

         // Find existing allocation for this account
         const existingAlloc = existingAllocations.find(
           (a) => a.accountId === account.id
         );

         const allocationAmount = alloc?.allocationAmount || 0;
         const allocatedPercent = alloc?.allocatedPercent || 0;

         if (existingAlloc) {
           // Update existing allocation record with new values
           updatedAllocations.push({
             ...existingAlloc,
             initialContribution: allocationAmount,
             allocatedPercent: allocatedPercent,
             allocationAmount: allocationAmount,
             percentAllocation: 0,
           } as GoalAllocation);
         } else {
           // Create new allocation if it doesn't exist
           updatedAllocations.push({
             id: `${goal.id}-${account.id}-${Date.now()}`,
             goalId: goal.id,
             accountId: account.id,
             initialContribution: allocationAmount,
             allocatedPercent: allocatedPercent,
             allocationAmount: allocationAmount,
             percentAllocation: 0,
           } as GoalAllocation);
         }
       }

      await onSubmit(updatedAllocations);

      // Wait a moment for backend to process, then refetch queries
      await new Promise(resolve => setTimeout(resolve, 500));

      // Refetch to ensure fresh data is immediately available
      await Promise.all([
        queryClient.refetchQueries({ queryKey: [QueryKeys.GOALS_ALLOCATIONS] }),
        queryClient.refetchQueries({ queryKey: [QueryKeys.GOALS] }),
        queryClient.refetchQueries({ queryKey: ["historicalValuation"] }),
      ]);

      handleOpenChange(false);
    } catch (err) {
      toast.error(t("editAllocationsModal.saveFailed"), {
        description: err instanceof Error ? err.message : t("editAllocationsModal.unknownError"),
      });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-[800px] max-h-[90vh] flex flex-col bg-background/95 backdrop-blur-xl">
        <DialogHeader>
          <DialogTitle className="text-xl font-bold">{t("editModal.title", { title: goal.title })}</DialogTitle>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto space-y-6 py-2 px-1">
          {/* Summary Card */}
          <Card className="bg-primary/5 border-primary/20 shadow-none">
            <CardContent className="p-4 flex flex-col sm:flex-row gap-4 justify-between items-start sm:items-center">
              <div>
                <h4 className="text-sm font-semibold text-primary mb-1">{t("editAllocationsModal.summary")}</h4>
                <p className="text-xs text-muted-foreground">
                  {t("editAllocationsModal.reviewAllocations")}
                </p>
              </div>
              <div className="flex flex-col items-end gap-1">
                {Object.values(allocations).some(a => (a.allocationAmount > 0 || a.allocatedPercent > 0)) && (
                  <Badge variant="outline" className="bg-background text-primary border-primary/30">
                    {t("editAllocationsModal.totalAllocated")}: {Object.entries(allocations).filter(([_, v]) => v.allocationAmount > 0 || v.allocatedPercent > 0).length} {t("editAllocationsModal.accounts")}
                  </Badge>
                )}
              </div>
            </CardContent>
          </Card>

          <div className="space-y-4">
            {accounts.map((account) => {
              const alloc = allocations[account.id];
              const baseUnallocated = availableBalances[account.id] ?? 0;
              const hasError = errors[account.id];

              // Calculate what's already allocated to other goals (percentage)
              const otherGoalsPercent = calculateOtherGoalsPercentage(
                account.id,
                goal.id,
                goal.startDate,
                goal.dueDate,
                allAllocations,
                isGoalAchieved
              );

              // Current user input for this goal
              const currentInputAmount = alloc?.allocationAmount || 0;
              const currentInputPercent = alloc?.allocatedPercent || 0;

              // Remaining unallocated after user's current input
              const remainingUnallocatedBalance = Math.max(0, baseUnallocated - currentInputAmount);
              const remainingUnallocatedPercent = Math.max(0, 100 - otherGoalsPercent - currentInputPercent);

              // Get currency symbol
              const currencySymbol = (0).toLocaleString('en-US', { style: 'currency', currency: account.currency, minimumFractionDigits: 0 }).replace(/\d/g, '').trim();

              return (
                <Card
                  key={account.id}
                  className={cn(
                    "transition-all duration-200 border-muted hover:border-primary/50 hover:shadow-md",
                    hasError && "border-destructive/50 bg-destructive/5"
                  )}
                >
                  <CardContent className="p-4">
                    <div className="flex flex-col md:flex-row gap-6">

                      {/* Left: Account Info */}
                      <div className="flex-1 space-y-3">
                        <div className="flex items-center justify-between">
                          <div className="font-semibold text-base flex items-center gap-2">
                            {account.name}
                          </div>
                          <Badge variant="secondary" className="font-mono text-xs">{account.currency}</Badge>
                        </div>

                        <div className="grid grid-cols-2 gap-2 text-sm">
                          <span className="text-muted-foreground text-xs">{t("editAllocationsModal.unallocatedBalance")}</span>
                          <span className="text-muted-foreground text-xs text-right">{t("editAllocationsModal.unallocatedPercent")}</span>

                          <span className={cn(
                            "font-mono font-medium",
                            remainingUnallocatedBalance < baseUnallocated * 0.1 ? "text-amber-500" : "text-foreground"
                          )}>
                            {formatAmount(remainingUnallocatedBalance, account.currency, true)}
                          </span>
                          <span className={cn(
                            "font-mono font-medium text-right",
                            remainingUnallocatedPercent < 10 ? "text-amber-500" : "text-green-600"
                          )}>
                            {remainingUnallocatedPercent.toFixed(1)}%
                          </span>
                        </div>
                      </div>

                      <Separator orientation="vertical" className="hidden md:block h-auto bg-border/50" />
                      <Separator orientation="horizontal" className="md:hidden bg-border/50" />

                      {/* Right: Inputs */}
                      <div className="flex-1 grid grid-cols-2 gap-4 items-start">
                        <div className="space-y-2">
                          <Label className="text-xs text-muted-foreground uppercase tracking-wider">
                            {t("editAllocationsModal.amount")}
                          </Label>
                          <div className="relative">
                            <div className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none">
                              {currencySymbol || "$"}
                            </div>
                            <Input
                              type="number"
                              value={alloc?.allocationAmount ?? ""}
                              onChange={(e) =>
                                handleAmountChange(account.id, Number(e.target.value))
                              }
                              placeholder="0.00"
                              step="0.01"
                              min="0"
                              disabled={isLoading || isFetchingHistory}
                              className={cn(
                                "pl-7 font-mono",
                                hasError && "border-destructive focus-visible:ring-destructive"
                              )}
                            />
                          </div>
                        </div>

                        <div className="space-y-2">
                          <Label className="text-xs text-muted-foreground uppercase tracking-wider">
                            {t("editAllocationsModal.allocatedPercent")}
                          </Label>
                          <div className="relative">
                            <Input
                              type="number"
                              value={alloc?.allocatedPercent ?? ""}
                              onChange={(e) =>
                                handlePercentageChange(account.id, Number(e.target.value))
                              }
                              placeholder="0"
                              step="0.1"
                              min="0"
                              max="100"
                              disabled={isLoading || isFetchingHistory}
                              className={cn(
                                "pr-8 font-mono",
                                hasError && "border-destructive focus-visible:ring-destructive"
                              )}
                            />
                            <div className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none">
                              <Percent className="h-3 w-3" />
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>

                    {hasError && (
                      <div className="mt-3 text-xs font-medium text-destructive bg-destructive/10 p-2 rounded flex items-center animate-in fade-in slide-in-from-top-1">
                        ⚠️ {hasError}
                      </div>
                    )}
                  </CardContent>
                </Card>
              );
            })}
          </div>
        </div>

        <DialogFooter className="gap-2 sm:gap-0 mt-2">
          <Button variant="outline" onClick={() => handleOpenChange(false)} disabled={isLoading || isFetchingHistory}>
            {t("editAllocationsModal.cancel")}
          </Button>
          <Button onClick={handleSubmit} disabled={isLoading || isFetchingHistory} className="bg-primary text-primary-foreground hover:bg-primary/90">
            {isLoading ? t("editAllocationsModal.updating") : isFetchingHistory ? t("editAllocationsModal.loadingData") : t("editAllocationsModal.updateAllocations")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
