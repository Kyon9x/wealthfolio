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
import { Account, Goal, GoalAllocation } from "@/lib/types";
import { cn } from "@/lib/utils";
import { formatAmount } from "@/ui";
import { Percent } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { calculateOtherGoalsPercentage, useAllocationForm } from "../hooks/use-allocation-form";

interface EditSingleAllocationModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  goal: { id: string; title: string; startDate?: string; dueDate?: string };
  account: Account;
  currentAllocation?: GoalAllocation;
  currentAccountValue: number;
  allAllocations?: GoalAllocation[]; // All allocations for time-aware calculation
  allGoals?: Goal[]; // All goals for checking isAchieved status
  onSubmit: (allocation: GoalAllocation) => Promise<void>;
}

export function EditSingleAllocationModal({
  open,
  onOpenChange,
  goal,
  account,
  currentAllocation,
  currentAccountValue,
  allAllocations = [],
  allGoals = [],
  onSubmit,
}: EditSingleAllocationModalProps) {
  const { t } = useTranslation("goals");
  const [amount, setAmount] = useState<number>(currentAllocation?.initialContribution || 0);
  const [percentage, setPercentage] = useState<number>(currentAllocation?.allocatedPercent || 0);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [isLoading, setIsLoading] = useState(false);

  // Memoize these to prevent infinite re-renders in the hook
  const accountsArray = useMemo(() => [account], [account]);
  const currentAccountValuesMap = useMemo(
    () => new Map([[account.id, currentAccountValue]]),
    [account.id, currentAccountValue]
  );

  // Use the shared allocation form hook
  const {
    availableBalances,
    isFetchingHistory,
    isGoalAchieved,
  } = useAllocationForm({
    goal: goal as Goal,
    accounts: accountsArray,
    currentAccountValues: currentAccountValuesMap,
    allAllocations,
    allGoals,
    open,
  });

  // Reset state when opening (in case props changed)
  useEffect(() => {
    if (!open) return;
    setAmount(currentAllocation?.initialContribution || 0);
    setPercentage(currentAllocation?.allocatedPercent || 0);
    setErrors({});
  }, [open, currentAllocation]);

  // Handle amount change - INDEPENDENT from percentage
  const handleAmountChange = (value: number) => {
    setAmount(value);
    setErrors((prev) => ({ ...prev, amount: "" }));
  };

  // Handle percentage change - INDEPENDENT from amount
  const handlePercentageChange = (value: number) => {
    setPercentage(value);
    setErrors((prev) => ({ ...prev, percentage: "" }));
  };

  // Get base unallocated balance from hook
  const baseUnallocatedBalance = availableBalances[account.id] ?? currentAccountValue;

  // Calculate remaining unallocated after current input
  const remainingUnallocatedBalance = Math.max(0, baseUnallocatedBalance - amount);

  // Calculate unallocated percentage using shared utility
  const otherGoalsPercent = calculateOtherGoalsPercentage(
    account.id,
    goal.id,
    goal.startDate,
    goal.dueDate,
    allAllocations,
    isGoalAchieved
  );
  const remainingUnallocatedPercent = Math.max(0, 100 - otherGoalsPercent - percentage);

  // Validate form
  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (amount < 0) {
      newErrors.amount = t("singleAllocationModal.errors.amountNegative");
    }

    if (percentage < 0 || percentage > 100) {
      newErrors.percentage = t("singleAllocationModal.errors.percentageRange");
    }

    // Check if amount exceeds unallocated balance
    if (amount > baseUnallocatedBalance) {
      newErrors.amount = t("singleAllocationModal.errors.amountExceedsUnallocated");
    }

    // Check total percentage
    if (100 - otherGoalsPercent - percentage < 0) {
      newErrors.percentage = t("singleAllocationModal.errors.percentageExceedsTotal");
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  // Handle submit
  const handleSubmit = async () => {
    if (!validateForm()) {
      return;
    }

    setIsLoading(true);
    try {
      const allocation: GoalAllocation = {
        id: currentAllocation?.id || `${goal.id}-${account.id}-${Date.now()}`,
        goalId: goal.id,
        accountId: account.id,
        initialContribution: amount,
        allocatedPercent: percentage,
        allocationDate: currentAllocation?.allocationDate,
        startDate: currentAllocation?.startDate,
        endDate: currentAllocation?.endDate,
        percentAllocation: percentage,
        allocationAmount: amount,
      } as GoalAllocation;

      await onSubmit(allocation);
      onOpenChange(false);
      setAmount(0);
      setPercentage(0);
    } catch (err) {
      toast.error(t("singleAllocationModal.saveFailed"), {
        description: err instanceof Error ? err.message : t("singleAllocationModal.unknownError"),
      });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[800px] flex flex-col bg-background/95 backdrop-blur-xl">
        <DialogHeader>
          <DialogTitle className="text-xl font-bold">
            {currentAllocation ? t("singleAllocationModal.editTitle") : t("singleAllocationModal.createTitle")}
          </DialogTitle>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto space-y-6 py-2 px-1">
          {/* Summary Card */}
          <Card className="bg-primary/5 border-primary/20 shadow-none">
             <CardContent className="p-4 flex flex-col sm:flex-row gap-4 justify-between items-start sm:items-center">
              <div>
                <h4 className="text-sm font-semibold text-primary mb-1">{t("singleAllocationModal.summary")}</h4>
                <p className="text-xs text-muted-foreground">
                   {t("singleAllocationModal.reviewAllocation")}
                </p>
              </div>
              <div className="flex flex-col items-end gap-1">
                  <Badge variant="outline" className="bg-background text-primary border-primary/30">
                    {goal.title}
                  </Badge>
              </div>
            </CardContent>
          </Card>

          <Card
            className={cn(
              "transition-all duration-200 border-muted hover:border-primary/50 hover:shadow-md",
              (errors.amount || errors.percentage) && "border-destructive/50 bg-destructive/5"
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
                        remainingUnallocatedBalance < baseUnallocatedBalance * 0.1 ? "text-amber-500" : "text-foreground"
                      )}>
                        {isFetchingHistory ? "..." : formatAmount(remainingUnallocatedBalance, account.currency, true)}
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
                 <div className="flex-1 flex flex-col justify-end">
                   <div className="grid grid-cols-2 gap-4 text-sm">
                     {/* Labels row */}
                     <Label className="text-xs text-muted-foreground uppercase tracking-wider">
                       {t("singleAllocationModal.amount")}
                     </Label>
                     <Label className="text-xs text-muted-foreground uppercase tracking-wider">
                       {t("singleAllocationModal.allocationPercentage")}
                     </Label>

                     {/* Inputs row */}
                     <div className="relative">
                       <div className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none">
                         {(0).toLocaleString('en-US', { style: 'currency', currency: account.currency, minimumFractionDigits: 0 }).replace(/\d/g, '').trim() || "$"}
                       </div>
                       <Input
                         type="number"
                         value={amount}
                         onChange={(e) => handleAmountChange(Number(e.target.value))}
                         placeholder="0.00"
                         step="0.01"
                         min="0"
                         disabled={isLoading || isFetchingHistory}
                         className={cn(
                           "pl-7 font-mono",
                           errors.amount && "border-destructive focus-visible:ring-destructive"
                         )}
                       />
                     </div>
                     <div className="relative">
                       <Input
                         type="number"
                         value={percentage}
                         onChange={(e) => handlePercentageChange(Number(e.target.value))}
                         placeholder="0"
                         step="0.1"
                         min="0"
                         max="100"
                         disabled={isLoading || isFetchingHistory}
                         className={cn(
                            "pr-8 font-mono",
                            errors.percentage && "border-destructive focus-visible:ring-destructive"
                         )}
                       />
                       <div className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none">
                         <Percent className="h-3 w-3" />
                       </div>
                     </div>
                   </div>
                 </div>
               </div>

               {(errors.amount || errors.percentage) && (
                  <div className="mt-3 text-xs font-medium text-destructive bg-destructive/10 p-2 rounded flex items-center animate-in fade-in slide-in-from-top-1">
                    ⚠️ {errors.amount || errors.percentage}
                  </div>
               )}
            </CardContent>
          </Card>

          {/* Projected Value Info */}
           <div className="rounded-lg bg-blue-50/50 border border-blue-100 p-4">
            <div className="flex justify-between items-center mb-1">
              <p className="text-xs font-semibold text-blue-900">{t("singleAllocationModal.projectedContributionValue")}</p>
              <p className="font-mono font-bold text-blue-700">
                {formatAmount(
                  amount + Math.max(0, (currentAccountValue - amount) * (percentage / 100)),
                  account.currency,
                  true
                )}
              </p>
            </div>
            <p className="text-xs text-blue-600/80 text-right">
              {t("singleAllocationModal.initial")}: {formatAmount(amount, account.currency, false)} + {t("singleAllocationModal.growth")}: {formatAmount(
                Math.max(0, (currentAccountValue - amount) * (percentage / 100)),
                account.currency,
                false
              )}
            </p>
          </div>
        </div>

        <DialogFooter className="gap-2 sm:gap-0 mt-2">
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={isLoading || isFetchingHistory}>
            {t("singleAllocationModal.cancel")}
          </Button>
          <Button onClick={handleSubmit} disabled={isLoading || isFetchingHistory} className="bg-primary text-primary-foreground hover:bg-primary/90">
            {isLoading ? t("singleAllocationModal.saving") : isFetchingHistory ? t("singleAllocationModal.loading") : t("singleAllocationModal.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
