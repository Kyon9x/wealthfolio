import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useSettingsContext } from "@/lib/settings-provider";
import { Account, Goal, GoalAllocation } from "@/lib/types";
import { Icons } from "@/ui";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { EditSingleAllocationModal } from "./edit-single-allocation-modal";

interface AllocationHistoryTableProps {
  goalId: string;
  goalStartDate?: string;
  goalDueDate?: string;
  allocations: GoalAllocation[];
  allAllocations?: GoalAllocation[]; // All allocations across all goals for time-aware calculation
  allGoals?: Goal[]; // All goals for checking isAchieved status
  accounts: Map<string, Account>;
  currentAccountValues: Map<string, number>;
  onAllocationUpdated: (allocation: GoalAllocation) => Promise<void>;
  onAllocationDeleted?: (allocationId: string) => Promise<void>;
  readOnly?: boolean;
  getAllocationValue?: (allocationId: string) => number | undefined;
}

export function AllocationHistoryTable({
  goalId,
  goalStartDate,
  goalDueDate,
  allocations,
  allAllocations = [],
  allGoals = [],
  accounts,
  currentAccountValues,
  onAllocationUpdated,
  onAllocationDeleted,
  readOnly = false,
  getAllocationValue,
}: AllocationHistoryTableProps) {
  const [editingAllocationId, setEditingAllocationId] = useState<string | null>(null);
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);
  const [allocationToDelete, setAllocationToDelete] = useState<GoalAllocation | null>(null);
  const [allocationToReset, setAllocationToReset] = useState<GoalAllocation | null>(null);
  const { t } = useTranslation("goals");
  const { settings } = useSettingsContext();

  const { currencySymbol, formatCurrency } = useMemo(() => {
    const currency = settings?.baseCurrency || "USD";
    const formatter = new Intl.NumberFormat(undefined, {
      style: "currency",
      currency,
      currencyDisplay: "narrowSymbol",
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    });

    const parts = formatter.formatToParts(0);
    const symbolPart = parts.find((part) => part.type === "currency");
    const symbol = symbolPart?.value ?? "$";

    const formatValue = (value: number) => {
      const formatted = formatter.format(value);
      // Remove the currency symbol and trim whitespace
      return formatted.replace(symbol, "").trim();
    };

    return { currencySymbol: symbol, formatCurrency: formatValue };
  }, [settings?.baseCurrency]);

  const currentAllocation = allocations.find((a) => a.id === editingAllocationId);

  const handleResetAllocation = async (allocation: GoalAllocation) => {
    const resetAllocation: GoalAllocation = {
      ...allocation,
      initialContribution: 0,
      allocatedPercent: 0,
    };
    await onAllocationUpdated(resetAllocation);
    setAllocationToReset(null);
  };

  return (
    <div className="space-y-4">
      {/* Current Allocations Table */}
      <div>
        <h3 className="font-semibold text-sm mb-3">{t("allocationHistoryTable.currentAllocations")}</h3>
        <div className="border rounded-lg overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b bg-muted">
                <th className="text-left px-4 py-2">{t("allocationHistoryTable.account")}</th>
                <th className="text-right px-4 py-2">{t("allocationHistoryTable.initialContribution")}</th>
                <th className="text-right px-4 py-2">{t("allocationHistoryTable.allocatedRate")}</th>
                <th className="text-right px-4 py-2">{t("allocationHistoryTable.contributedValue")}</th>
                {!readOnly && <th className="text-center px-4 py-2">{t("allocationHistoryTable.actions")}</th>}
              </tr>
            </thead>
            <tbody>
              {allocations.map((alloc) => {
                const account = accounts.get(alloc.accountId);
                // Handle incomplete data or legacy field names
                const initialContribution = alloc.initialContribution;
                const allocatedPercent = alloc.allocatedPercent;

                // Calculate contributed value using the provided function
                const contributedValue = getAllocationValue?.(alloc.id) ?? 0;

                return (
                  <React.Fragment key={alloc.id}>
                    <tr className="border-b hover:bg-muted/50">
                      <td className="px-4 py-2">{account?.name}</td>
                      <td className="text-right px-4 py-2">
                        {currencySymbol}{formatCurrency(initialContribution)}
                      </td>
                      <td className="text-right px-4 py-2">
                        {allocatedPercent.toFixed(1)}%
                      </td>
                      <td className="text-right px-4 py-2 font-semibold">
                        {currencySymbol}{formatCurrency(contributedValue)}
                      </td>
                      {!readOnly && (
                        <td className="text-center px-4 py-2">
                          <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                              <Button
                                size="sm"
                                variant="outline"
                              >
                                <Icons.MoreVertical className="h-4 w-4" />
                              </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end">
                              <DropdownMenuItem
                                onClick={() => {
                                  setEditingAllocationId(alloc.id);
                                  setIsEditModalOpen(true);
                                }}
                              >
                                {t("operations.edit")}
                              </DropdownMenuItem>
                              <DropdownMenuItem
                                onClick={() => setAllocationToReset(alloc)}
                              >
                                {t("allocationHistoryTable.reset")}
                              </DropdownMenuItem>
                              <DropdownMenuItem
                                className="text-red-600"
                                onClick={() => setAllocationToDelete(alloc)}
                              >
                                {t("operations.delete")}
                              </DropdownMenuItem>
                            </DropdownMenuContent>
                          </DropdownMenu>
                        </td>
                      )}
                    </tr>
                  </React.Fragment>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>

      {/* Edit Modal */}
      {currentAllocation && (
        <EditSingleAllocationModal
          open={isEditModalOpen}
          onOpenChange={setIsEditModalOpen}
          goal={{ id: goalId, title: "", startDate: goalStartDate, dueDate: goalDueDate }}
          account={accounts.get(currentAllocation.accountId) || { id: "", name: "" } as Account}
          currentAllocation={currentAllocation}
          currentAccountValue={currentAccountValues.get(currentAllocation.accountId) || 0}
          allAllocations={allAllocations}
          allGoals={allGoals}
          onSubmit={onAllocationUpdated}
        />
      )}

      {/* Reset Confirmation Modal */}
      <AlertDialog
        open={!!allocationToReset}
        onOpenChange={(open) => !open && setAllocationToReset(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("resetAllocation.title")}</AlertDialogTitle>
            <AlertDialogDescription>
              {t("resetAllocation.description")}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("editModal.cancel")}</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                if (allocationToReset) {
                  handleResetAllocation(allocationToReset);
                }
              }}
            >
              {t("resetAllocation.confirm")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Delete Confirmation Modal */}
      <AlertDialog
        open={!!allocationToDelete}
        onOpenChange={(open) => !open && setAllocationToDelete(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("deleteAllocation.title")}</AlertDialogTitle>
            <AlertDialogDescription>
              {t("deleteAllocation.description")}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("editModal.cancel")}</AlertDialogCancel>
            <AlertDialogAction
              className="bg-red-600 hover:bg-red-700 text-white"
              onClick={async () => {
                if (allocationToDelete && onAllocationDeleted) {
                  await onAllocationDeleted(allocationToDelete.id);
                  setAllocationToDelete(null);
                }
              }}
            >
              {t("deleteAllocation.confirm")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
