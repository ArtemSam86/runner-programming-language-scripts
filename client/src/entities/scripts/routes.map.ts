import {RUN_NAME, SCRIPTS_NAME} from "@/shared/constants.ts";

interface ScriptsRoutesMap {
    listScripts: string;
    runScripts: string;
}

export const scriptsRoutesMap: ScriptsRoutesMap = {
    listScripts: `/${SCRIPTS_NAME}`,
    runScripts: `/${RUN_NAME}`,
}