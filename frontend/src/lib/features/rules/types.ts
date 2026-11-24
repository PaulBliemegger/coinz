export type RuleOperator = 'contains' | 'contains_word' | 'equals';
export interface RuleDefinition {
  name: string;
  operator: RuleOperator;
  pattern: string;
  source_id: string | null;
  category_id: string | null;
  label_ids: string[];
  enabled: boolean;
  revision?: number;
}
export interface LabelRule extends RuleDefinition {
  id: string;
  revision: number;
  pause_reason: string | null;
}
export interface RuleSnapshot {
  revision: number;
  rules: LabelRule[];
}
export interface RuleSeed {
  description: string;
  category_id: string | null;
  label_ids: string[];
}
export interface RuleSuggestions {
  revision: number;
  category_id: string | null;
  category_candidates: string[];
  label_ids: string[];
  conflict: boolean;
  matched_rules: {
    id: string;
    name: string;
    operator: RuleOperator;
    pattern: string;
    category_id: string | null;
    label_ids: string[];
  }[];
}
export interface RuleExample {
  id: string;
  description: string;
  date: string;
  amount_cents: string;
}
export const operatorNames: Record<RuleOperator, string> = {
  contains: 'contains',
  contains_word: 'contains word',
  equals: 'equals',
};
