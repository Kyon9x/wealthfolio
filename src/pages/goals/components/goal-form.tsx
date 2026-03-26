import { zodResolver } from "@hookform/resolvers/zod";
import { useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { toast } from "sonner";
import * as z from "zod";

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
import { Switch } from "@/components/ui/switch";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { DatePickerInput, Icons, MoneyInput } from "@/ui";

import {
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";


import { newGoalSchema } from "@/lib/schemas";
import { useGoalMutations } from "@/pages/goals/hooks/use-goal-mutations";

// Infer type from schema (input type = works with Date)
type NewGoal = z.infer<typeof newGoalSchema>;

interface GoalFormProps {
  defaultValues?: Partial<NewGoal>;
  onSuccess?: () => void;
}

export function GoalForm({ defaultValues, onSuccess = () => undefined }: GoalFormProps) {
  const { t } = useTranslation("goals");
  const navigate = useNavigate();
  const { addGoalMutation, updateGoalMutation } = useGoalMutations();

  const form = useForm<NewGoal>({
    resolver: zodResolver(newGoalSchema),
    defaultValues: {
      id: defaultValues?.id,
      title: defaultValues?.title || "",
      description: defaultValues?.description || "",
      targetAmount: defaultValues?.targetAmount || 0,
      targetReturnRate: defaultValues?.targetReturnRate,
      startDate: defaultValues?.startDate,
      dueDate: defaultValues?.dueDate,
      monthlyInvestment: defaultValues?.monthlyInvestment,
      isAchieved: defaultValues?.isAchieved || false,
    },
  });

  // Auto-calculate monthly investment when target amount, return rate, or dates change
  useEffect(() => {
    const subscription = form.watch((values) => {
      const { targetAmount, targetReturnRate, startDate, dueDate } = values;

      if (!targetAmount || targetReturnRate === undefined || !startDate || !dueDate) {
        return;
      }

      const start = startDate instanceof Date ? startDate : new Date(startDate);
      const end = dueDate instanceof Date ? dueDate : new Date(dueDate);

      // Calculate months between dates
      const monthsDiff =
        (end.getFullYear() - start.getFullYear()) * 12 + (end.getMonth() - start.getMonth());

      if (monthsDiff <= 0) {
        return;
      }

      // Calculate monthly investment
      // PMT = Target / [((1 + r)^n - 1) / r]
      const monthlyRate = targetReturnRate / 100 / 12;
      let monthlyInvestment: number;

      if (monthlyRate === 0) {
        monthlyInvestment = targetAmount / monthsDiff;
      } else {
        const compoundFactor = Math.pow(1 + monthlyRate, monthsDiff);
        monthlyInvestment = targetAmount / ((compoundFactor - 1) / monthlyRate);
      }

      // Format to 2 decimal places
      monthlyInvestment = Math.round(monthlyInvestment * 100) / 100;

      form.setValue("monthlyInvestment", monthlyInvestment, { shouldDirty: true });
    });

    return () => subscription.unsubscribe();
  }, [form]);

  // State for start date change confirmation dialog
  const [showStartDateConfirm, setShowStartDateConfirm] = useState(false);
  const [pendingPayload, setPendingPayload] = useState<any>(null);

  // Helper to check if start date has changed
  const hasStartDateChanged = (newStartDate: Date | string | undefined): boolean => {
    const originalStartDate = defaultValues?.startDate;
    if (!originalStartDate || !newStartDate) return false;

    const originalDate = originalStartDate instanceof Date ? originalStartDate : new Date(originalStartDate);
    const newDate = newStartDate instanceof Date ? newStartDate : new Date(newStartDate);

    return originalDate.toDateString() !== newDate.toDateString();
  };

  // Execute the actual update mutation
  const executeUpdate = (payload: any) => {
    updateGoalMutation.mutate(payload, {
      onSuccess: () => {
        onSuccess();
      }
    });
  };

  // Handle confirmed start date change
  const handleConfirmStartDateChange = () => {
    setShowStartDateConfirm(false);
    if (pendingPayload) {
      executeUpdate(pendingPayload);
      setPendingPayload(null);
    }
  };

  function onSubmit(data: NewGoal) {
    const { startDate, dueDate, ...rest } = data;
    // Convert dates to ISO strings for the backend
    const payload = {
      ...rest,
      startDate: startDate instanceof Date ? startDate.toISOString() : startDate,
      dueDate: dueDate instanceof Date ? dueDate.toISOString() : dueDate,
    };

    if (rest.id) {
      // Check if start date is changing for an existing goal
      if (hasStartDateChanged(startDate)) {
        // Show confirmation dialog before proceeding
        setPendingPayload(payload);
        setShowStartDateConfirm(true);
        return;
      }

      // No start date change, proceed normally
      return executeUpdate(payload);
    }

    return addGoalMutation.mutate(payload, {
      onSuccess: (createdGoal) => {
        onSuccess();

        // Show toast with action to add allocations
        toast.success(t("form.postCreation.success"), {
          description: t("form.postCreation.description"),
          action: {
            label: t("form.postCreation.addAllocations"),
            onClick: () => navigate(`/goals/${createdGoal.id}`),
          },
          duration: 8000,
        });
      },
    });
  }

  return (
    <>
      {/* Confirmation dialog for start date change */}
      <AlertDialog open={showStartDateConfirm} onOpenChange={setShowStartDateConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("form.startDateChanged.confirmTitle")}</AlertDialogTitle>
            <AlertDialogDescription>
              {t("form.startDateChanged.confirmDescription")}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={() => setPendingPayload(null)}>
              {t("form.buttons.cancel")}
            </AlertDialogCancel>
            <AlertDialogAction onClick={handleConfirmStartDateChange}>
              {t("form.startDateChanged.confirmButton")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
          <DialogHeader>
          <DialogTitle>
            {defaultValues?.id ? t("form.updateTitle") : t("form.addTitle")}
          </DialogTitle>
          <DialogDescription>
            {defaultValues?.id ? t("form.updateDescription") : t("form.addDescription")}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6">
          {/* Hidden ID field */}
          <input type="hidden" name="id" />

          {/* Goal Name - Full Width */}
          <FormField
            control={form.control}
            name="title"
            render={({ field }) => (
              <FormItem>
                <FormLabel>{t("form.fields.title.label")}</FormLabel>
                <FormControl>
                  <Input placeholder={t("form.fields.title.placeholder")} {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          {/* Description - Full Width */}
          <FormField
            control={form.control}
            name="description"
            render={({ field }) => (
              <FormItem>
                <FormLabel>{t("form.fields.description.label")}</FormLabel>
                <FormControl>
                  <textarea
                    placeholder={t("form.fields.description.placeholder")}
                    className="border-input bg-background ring-offset-background placeholder:text-muted-foreground focus-visible:ring-ring flex min-h-24 w-full rounded-md border px-3 py-2 text-sm focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
                    {...field}
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          {/* Target Amount - Full Width */}
          <FormField
            control={form.control}
            name="targetAmount"
            render={({ field }) => (
              <FormItem>
                <FormLabel>{t("form.fields.targetAmount.label")}</FormLabel>
                <FormControl>
                  <MoneyInput placeholder={t("form.fields.targetAmount.placeholder")} {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          {/* Start Date and Due Date - 2 Column Grid */}
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
            <FormField
              control={form.control}
              name="startDate"
              render={({ field }) => (
                <FormItem>
                  <TooltipProvider>
                    <Tooltip delayDuration={300}>
                      <TooltipTrigger asChild>
                        <FormLabel className="cursor-help">
                          {t("form.fields.startDate.label")}
                        </FormLabel>
                      </TooltipTrigger>
                      <TooltipContent side="top" align="start">
                        <div className="max-w-xs">{t("form.fields.startDate.description")}</div>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                  <FormControl>
                    <DatePickerInput
                      value={field.value}
                      onChange={field.onChange}
                      className="w-full"
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="dueDate"
              render={({ field }) => (
                <FormItem>
                  <TooltipProvider>
                    <Tooltip delayDuration={300}>
                      <TooltipTrigger asChild>
                        <FormLabel className="cursor-help">
                          {t("form.fields.dueDate.label")}
                        </FormLabel>
                      </TooltipTrigger>
                      <TooltipContent side="top" align="start">
                        <div className="max-w-xs">{t("form.fields.dueDate.description")}</div>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                  <FormControl>
                    <DatePickerInput
                      value={field.value}
                      onChange={field.onChange}
                      className="w-full"
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
          </div>

          {/* Investment Plan Section */}
          <div className="space-y-4">
            <div className="flex items-center gap-2">
              <Icons.HandCoins className="text-primary h-5 w-5" />
              <h3 className="text-foreground text-lg font-medium">
                {t("form.fields.investmentPlan.title")}
              </h3>
            </div>

            <div className="bg-muted/50 border-border space-y-4 rounded-lg border p-4">
              {/* Target Return Rate */}
              <FormField
                control={form.control}
                name="targetReturnRate"
                render={({ field }) => (
                  <FormItem>
                    <TooltipProvider>
                      <Tooltip delayDuration={300}>
                        <TooltipTrigger asChild>
                          <FormLabel className="cursor-help">
                            {t("form.fields.targetReturnRate.label")}
                          </FormLabel>
                        </TooltipTrigger>
                        <TooltipContent side="top" align="start">
                          <div className="max-w-xs">{t("form.fields.targetReturnRate.hint")}</div>
                        </TooltipContent>
                      </Tooltip>
                    </TooltipProvider>
                    <FormControl>
                      <div className="relative">
                        <Input
                          type="number"
                          step="0.1"
                          min="0"
                          max="100"
                          placeholder={t("form.fields.targetReturnRate.placeholder")}
                          value={field.value ?? ""}
                          onChange={(e) => {
                            const value = e.target.value;
                            field.onChange(value === "" ? undefined : parseFloat(value));
                          }}
                          onBlur={field.onBlur}
                          name={field.name}
                          ref={field.ref}
                          className="no-spinner pr-16"
                        />
                        <span className="text-muted-foreground pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-sm">
                          {t("form.fields.targetReturnRate.suffix")}
                        </span>
                      </div>
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              {/* Monthly Investment (DCA) - Auto-calculated */}
              <FormField
                control={form.control}
                name="monthlyInvestment"
                render={({ field }) => (
                  <FormItem>
                    <TooltipProvider>
                      <Tooltip delayDuration={300}>
                        <TooltipTrigger asChild>
                          <FormLabel className="cursor-help">
                            {t("form.fields.monthlyInvestment.label")}
                          </FormLabel>
                        </TooltipTrigger>
                        <TooltipContent side="top" align="start">
                          <div className="max-w-xs">
                            {t("form.fields.monthlyInvestment.description")}
                          </div>
                        </TooltipContent>
                      </Tooltip>
                    </TooltipProvider>
                    <FormControl>
                      <MoneyInput
                        placeholder={t("form.fields.monthlyInvestment.placeholder")}
                        {...field}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>
          </div>

          {/* Goal Achieved Toggle - Only show in edit mode */}
          {defaultValues?.id ? (
            <FormField
              control={form.control}
              name="isAchieved"
              render={({ field }) => (
                <FormItem className="flex items-center gap-2">
                  <FormControl>
                    <Switch checked={field.value} onCheckedChange={field.onChange} />
                  </FormControl>
                  <FormLabel className="mt-0!">{t("form.fields.isAchieved.label")}</FormLabel>
                  <FormMessage />
                </FormItem>
              )}
            />
          ) : null}
        </div>

        <DialogFooter className="gap-2">
          <DialogTrigger asChild>
            <Button variant="outline">{t("form.buttons.cancel")}</Button>
          </DialogTrigger>
          <Button type="submit">
            <Icons.Save className="mr-2 h-4 w-4" />
            <span>{defaultValues?.id ? t("form.buttons.update") : t("form.buttons.add")}</span>
          </Button>
        </DialogFooter>
        </form>
      </Form>
    </>
  );
}
