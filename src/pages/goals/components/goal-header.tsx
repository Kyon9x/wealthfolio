/**
 * Goal Header Component
 * Displays goal title, status badge, and action buttons
 */

import type { Goal } from "@/lib/types";
import { Button, Icons } from "@/ui";
import { useTranslation } from "react-i18next";

interface GoalHeaderProps {
  goal: Goal;
  onEdit: () => void;
  onBack: () => void;
}

export function GoalHeader({ goal, onEdit, onBack }: GoalHeaderProps) {
  const { t } = useTranslation("goals");

  return (
    <div className="flex items-start justify-between border-b pb-6">
      <div>
        <div className="flex items-center gap-3">
          <h1 className="text-foreground text-2xl font-bold">
            {t("details.title", { title: goal.title })}
          </h1>
          {goal.isAchieved && (
            <span className="inline-flex items-center gap-1.5 rounded-full bg-green-100 px-3 py-1 text-sm font-medium text-green-700 dark:bg-green-900/30 dark:text-green-400">
              <Icons.Check className="h-4 w-4" />
              {t("completedGoal.badge")}
            </span>
          )}
        </div>
        <p className="text-muted-foreground mt-1">
          {goal.isAchieved
            ? t("completedGoal.description")
            : t("details.description", { title: goal.title })}
        </p>
      </div>
      <div className="flex gap-3">
        <Button variant="outline" onClick={onEdit}>
          <Icons.Pencil className="mr-2 h-4 w-4" />
          {t("details.editGoal")}
        </Button>
        <Button onClick={onBack}>
          <Icons.ArrowLeft className="mr-2 h-4 w-4" />
          {t("details.back")}
        </Button>
      </div>
    </div>
  );
}
