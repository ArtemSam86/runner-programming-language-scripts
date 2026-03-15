import {SCRIPTS_NAME} from "@/shared/constants.ts";

export interface ScriptRoutesMap {
    updateScript: (name: string) => string;
    createScript: string;
}

export const scriptRoutesMap: ScriptRoutesMap = {
    updateScript: (name: string): string => `/${SCRIPTS_NAME}/${name}`,
    createScript: `/${SCRIPTS_NAME}`,
};