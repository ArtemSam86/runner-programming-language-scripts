import HttpClient from "@/services/httpClient.ts";
import type {RunScriptResponse, ScriptRequestParams} from "@/entities/script/types.ts";
import {scriptRoutesMap} from "@/entities/script/routes.map.ts";
import type {RunScriptParams, ScriptMetadata} from "@/shared/types/common.ts";
import {mapRunnerScript} from "@/entities/mappers/script.map.ts";

const DEFAULT_PARAMS: RunScriptParams = {
    data: {},
}

export const updateScript = async (
    params: ScriptRequestParams
) => {
    const { name, ..._params } = params;
    const { data } = await HttpClient
        .put<ScriptMetadata>(scriptRoutesMap.updateScript(name), _params);

    return mapRunnerScript(data);
};

export const createScript = async (
    params: ScriptRequestParams
) => {
    await HttpClient.post(scriptRoutesMap.createScript, params);
};

export const deleteScript = async (
    name: string
) => {
    const { data } = await HttpClient.delete<void>(scriptRoutesMap.updateScript(name));
    return data;
};

export const runSingleScript = async (
    name: string
) => {
    try {
        const { data } = await HttpClient.post<RunScriptResponse>(
            '/run/' + name,
            DEFAULT_PARAMS
        );

        return data;
    } catch (error) {
        return Promise.resolve<RunScriptResponse>({
            exit_code: 0,
            stderr: String(error),
            stdout: "",
            timed_out: false
        });
    }

}