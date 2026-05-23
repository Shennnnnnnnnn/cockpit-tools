import type { TFunction } from 'i18next';
import type { CodexSessionVisibilityRepairSummary } from '../types/codex';

export function formatCodexSessionVisibilityRepairMessage(
  summary: CodexSessionVisibilityRepairSummary,
  t: TFunction,
): string {
  void t;
  return summary.message;
}
