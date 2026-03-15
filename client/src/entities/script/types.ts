import type {RunScriptResult} from "@/shared/types/common.ts";

export interface ScriptRequestParams {
    name: string;
    code?: string;
    description?: string;
    result?: string;
}

export type RunScriptResponse = RunScriptResult;