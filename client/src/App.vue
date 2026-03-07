<script setup lang="ts">
import HttpClient from './services/httpClient.ts';
import type {CodeEditorData} from './components/CodeEditor.vue';
import CodeEditor from './components/CodeEditor.vue';
import {computed, onMounted, ref} from 'vue';
import type {RunnerScript, Script} from "@/components/CodeEditor.vue";

interface SearchQueryName {
  type: 'name';
  query: string;
}
interface SearchQuerySize {
  type: 'size';
  query: number;
}
interface SearchQueryDate {
  type: 'date';
  query: Date;
}
type SearchQuery = SearchQueryName | SearchQuerySize | SearchQueryDate;

type CmdArgs = '--option' | '--verbose' | string;
interface RunScriptParams {
  data: Record<string, string | number>;
  args?: CmdArgs[];
}

const DEFAULT_PARAMS: RunScriptParams = {
  data: {},
}
const DEFAULT_CODE = `
import sys
import json

# NEW
def main():
    # Читаем всё из stdin
    raw_data = sys.stdin.read()
    if not raw_data:
        result = {"error": "No input provided"}
    else:
        try:
            data = json.loads(raw_data)
            # Здесь ваша логика
            result = {"received": data, "status": "ok"}
        except Exception as e:
            result = {"error": str(e)}

    # Выводим результат как JSON
    print(json.dumps(result))

if __name__ == "__main__":
    main()
`;

const runnerScripts = ref<RunnerScript[]>([]);
const isSearchScripts = ref<boolean>(false);
const editorCode = ref<string>(DEFAULT_CODE.trim());

const nameSelectScripts = computed(() =>
    runnerScripts.value.map((script) => script.isSelected && script.name).filter(Boolean));

/** MAPPERS **/
const mapRunnerScript = (script: Script): RunnerScript => {
  return {
    ...script,
    created: Intl.DateTimeFormat('ru-RU').format(new Date(script.created)),
    modified: Intl.DateTimeFormat('ru-RU').format(new Date(script.modified)),
    size: script.size, // Байт или Math.round((script.size / 1024) * 100) / 100, // В Кб
    isSelected: false,
    description: script.description || 'Нет описания',
    statusRun: {
      isCompleted: false,
      variant: 'secondary',
      loading: false,
    },
    scriptRunResult: '',
  };
};

/** ACTIONS **/
const getScripts = async (searchQuery?: SearchQuery) => {
  let url = '/scripts';
  if (searchQuery?.type === 'name') {
    const query = new URLSearchParams();
    query.append('query', searchQuery.query || '');
    url = `${url}?${query.toString()}`;
  }

  const response = await HttpClient.get<string>(url);
  const { data } = response;

  runnerScripts.value = JSON.parse(data).map(mapRunnerScript);
};
const handleDownloadScriptCode = async (name: string) => {
  const { data } = await HttpClient.get('/scripts/' + name);
  const codeEditorData: CodeEditorData = JSON.parse(data);
  editorCode.value = codeEditorData.code;
};
const handleCreateNewScript = async (params: CodeEditorData) => {
    await Promise.all([
      await HttpClient.post('/scripts', JSON.stringify(params)),
      await getScripts(),
    ])
};
const handleCreateNewScriptFromTemplate = async (params: CodeEditorData) => {
  await handleCreateNewScript(params);
};
const handleUpdateCurrentScript = async (params: CodeEditorData) => {
  await Promise.all([
    await HttpClient.put('/scripts/' + params.name, JSON.stringify({ code: params.code })),
    await getScripts(),
  ]);
};
const handleDeleteCurrentScript = async (name: string) => {
  await Promise.all([
    await HttpClient.delete('/scripts/' + name),
    await getScripts(),
  ])
}
const onSearchScript = async (value: string) => {
  isSearchScripts.value = true;
  console.log(value);
  await getScripts({ query: value, type: 'name' });
  isSearchScripts.value = false;
};
const updateDescription = async (name: string) => {
  const description = runnerScripts.value.find((script) => script.name === name)?.description;

  await Promise.all([
    await HttpClient.post(
        '/scripts/descriptions',
        JSON.stringify({ [name]: description })
    ),
    await getScripts(),
  ]);
};

const jsonParseStringify = (jsonStr: string) => {
  const parsedJson = JSON.parse(jsonStr);
  return JSON.stringify(parsedJson, null, 2)
};

/** RUNS **/
const handleRunCurrentScript = async (name: string) => {
  runnerScripts.value = runnerScripts.value
      .map((script) => script.name === name ? {
        ...script,
        isSelected: false,
        statusRun: {
          isCompleted: false,
          variant: 'secondary',
          loading: true
        }
      }
      : script);

  const response = await HttpClient.post(
      '/run/' + name,
      JSON.stringify(DEFAULT_PARAMS)
  );
  const { data } = response;
  console.log(response);

  const dataStdout = JSON.parse(data).stdout;

  runnerScripts.value = runnerScripts.value
      .map((script) => script.name === name ? {
        ...script,
        statusRun: {
          isCompleted: !!dataStdout,
          variant: !!dataStdout ? 'default' : 'secondary',
          loading: false,
        },
        scriptRunResult: dataStdout ? jsonParseStringify(dataStdout) : '',
      }
      : script);
}
const runSelectedScripts = async () => {
  runnerScripts.value = runnerScripts.value
      .map((script) => nameSelectScripts.value.includes(script.name)
          ? {
            ...script,
            statusRun: {
              isCompleted: false,
              variant: 'secondary',
              loading: true
            }
          }
          : script);

  const { data } = await HttpClient.post(
      `/run?names=${nameSelectScripts.value.join(',')}`,
      JSON.stringify(DEFAULT_PARAMS)
  );

  let dataResults = JSON.parse(data).results;
  let results = { ...JSON.parse(data).results };
  Object.keys(dataResults).forEach((k) => {
    results = {
      ...results,
      [k]: results[k].stdout ? JSON.parse(results[k].stdout) : ''
    };
  });

  runnerScripts.value = runnerScripts.value
      .map((script) => nameSelectScripts.value.includes(script.name)
          ? {
            ...script,
            statusRun: {
              isCompleted: !!results[script.name],
              variant: !!results[script.name] ? 'default' : 'secondary',
              loading: false,
            },
            scriptRunResult: JSON.stringify(results[script.name], null, 2),
          }
          : script);

  console.log('>>>runnerScripts.value: ', runnerScripts.value);
}

onMounted(async () => {
  await getScripts();
});
</script>

<template>
    <CodeEditor
        v-model:isSearchScripts="isSearchScripts"
        v-model:runnerScripts="runnerScripts"
        v-model:editorCode="editorCode"
        :defaultCode="DEFAULT_CODE"
        @downloadScriptCode="handleDownloadScriptCode"
        @createNewScript="handleCreateNewScript"
        @createNewScriptFromTemplate="handleCreateNewScriptFromTemplate"
        @updateCurrentScript="handleUpdateCurrentScript"
        @deleteCurrentScript="handleDeleteCurrentScript"
        @runCurrentScript="handleRunCurrentScript"
        @runSelectedScripts="runSelectedScripts"
        @onSearchScript="onSearchScript"
        @updateDescription="updateDescription"
    />
</template>

<style scoped></style>
