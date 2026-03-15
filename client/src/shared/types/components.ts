export interface SaveScriptParams {
    name: string;
    code?: string;
    description?: string;
    result?: string;
}

export type Variant = 'secondary' | 'default' | 'destructive';
export type Status = 'SUCCESS' | 'ERROR' | 'NEUTRAL';
export type Statuses = Record<Variant, Status>;