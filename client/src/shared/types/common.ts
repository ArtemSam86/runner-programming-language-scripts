import type {Variant} from "@/shared/types/components.ts";

export interface ScriptMetadata {
    name: string;
    code: string;
    description: string | null;
    result: string | null;
    size: number;
    created: string;
    modified: string;
}

interface StatusRun {
    isCompleted: boolean;
    loading?: boolean;
    variant: Variant;
}

export interface RunnerScript extends ScriptMetadata {
    isSelected: boolean;
    result: string;
    description: string;
    statusRun: StatusRun;
}

export type SortOrder = 'asc' | 'desc';
export type SortBy = 'name' | 'size' | 'created' | 'modified';

export interface RunScriptResult {
    stdout: string;
    stderr: string;
    exit_code: number;
    timed_out: boolean;
}

type CmdArgs = '--option' | '--verbose' | string;
export interface RunScriptParams {
    data: Record<string, string | number>;
    args?: CmdArgs[];
}