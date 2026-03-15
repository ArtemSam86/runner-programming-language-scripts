import type {RunScriptResult, SortBy, SortOrder} from "@/shared/types/common.ts";

export interface ScriptsRequestParams {
    query?: string;
    sort_by?: SortBy;
    sort_order?: SortOrder;
}


export interface RunScriptsResponse {
    results: Record<string, RunScriptResult>;
}