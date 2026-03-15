import type {RunnerScript, ScriptMetadata} from "@/shared/types/common.ts";

export const mapRunnerScript = (script: ScriptMetadata): RunnerScript => {
    return {
        ...script,
        created: Intl.DateTimeFormat('ru-RU').format(new Date(script.created)),
        modified: Intl.DateTimeFormat('ru-RU').format(new Date(script.modified)),
        size: script.size, // Байт или Math.round((script.size / 1024) * 100) / 100, // В Кб
        isSelected: false,
        description: script.description || 'Нет описания',
        result: script.result || '',
        // front mixin
        statusRun: {
            isCompleted: false,
            variant: 'secondary',
            loading: false,
        },
    };
};