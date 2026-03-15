import HttpClient from "@/services/httpClient.ts";
import type {RunScriptsResponse, ScriptsRequestParams} from "@/entities/scripts/types.ts";
import type {RunScriptParams, ScriptMetadata} from "@/shared/types/common.ts";
import {scriptsRoutesMap} from "@/entities/scripts/routes.map.ts";
import {mapRunnerScript} from "@/entities/mappers/script.map.ts";

const defaultParams: ScriptsRequestParams = {
    sort_by: 'created',
    sort_order: 'desc',
}
const DEFAULT_PARAMS: RunScriptParams = {
    data: {},
}

export const getListScripts = async (
    params: ScriptsRequestParams = {}
) => {
    const response = await HttpClient
        .get<ScriptMetadata[]>(scriptsRoutesMap.listScripts, { params: { ...defaultParams, ...params } });
    const { data } = response;

    return data.map(mapRunnerScript);
};

export const runScripts = async (
    names: string[]
) => {
    const { data } = await HttpClient.post<RunScriptsResponse>(
        `${scriptsRoutesMap.runScripts}?names=${names.join(',')}`,
        DEFAULT_PARAMS
    );

    return data;
}