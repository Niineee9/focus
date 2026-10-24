import { invoke } from "@tauri-apps/api/core";

export interface Category {
  id: number;
  name: string;
  icon: string;
  color: string;
  daysMask: number;
  createdAt: string;
}

export interface CategoryInput {
  name: string;
  icon: string;
  color: string;
  daysMask: number;
}

export interface Completion {
  categoryId: number;
  date: string;
}

export type DayStatus = "done" | "missed" | "pending" | "off" | "extra" | "before" | "future";

export interface Progress {
  done: number;
  target: number;
}

export interface CategoryStats {
  categoryId: number;
  currentStreak: number;
  bestStreak: number;
  completionRate: number;
  scheduledDays: number;
  completedScheduled: number;
  extraCompletions: number;
  doneToday: boolean;
  scheduledToday: boolean;
  week: Progress;
  month: Progress;
  history: { date: string; status: DayStatus }[];
}

export interface Stats {
  today: string;
  overall: {
    perfectStreak: number;
    bestPerfectStreak: number;
    today: Progress;
    week: Progress;
    month: Progress;
    completionRate: number;
  };
  categories: CategoryStats[];
}

export const api = {
  listCategories: () => invoke<Category[]>("list_categories"),
  createCategory: (input: CategoryInput) => invoke<Category>("create_category", { input }),
  updateCategory: (id: number, input: CategoryInput) => invoke<void>("update_category", { id, input }),
  deleteCategory: (id: number) => invoke<void>("delete_category", { id }),
  getCompletions: (from: string, to: string) => invoke<Completion[]>("get_completions", { from, to }),
  toggleCompletion: (categoryId: number, date: string) =>
    invoke<boolean>("toggle_completion", { categoryId, date }),
  getStats: () => invoke<Stats>("get_stats"),
};
