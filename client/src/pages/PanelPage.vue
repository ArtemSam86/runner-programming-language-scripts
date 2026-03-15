<script setup lang="ts">
import {computed, onMounted, ref} from 'vue';
import PanelRunAndEditedScripts from '@/components/PanelRunAndEditedScripts.vue';
import {getListScripts, runScripts} from "@/entities/scripts";
import type {RunnerScript} from "@/shared/types/common.ts";
import {createScript, deleteScript, runSingleScript, updateScript} from "@/entities/script";
import {useAuth} from "@/composables/useAuth.ts";
import type {SaveScriptParams} from "@/shared/types/components.ts";
import type {ScriptsRequestParams} from "@/entities/scripts/types.ts";

const runnerScripts = ref<RunnerScript[]>([]);
const isSearchScripts = ref<boolean>(false);

const nameSelectScripts = computed(() =>
    runnerScripts.value.filter((script) => script.isSelected && script.name).map(script => script.name));

const { logout, user } = useAuth();

/** ACTIONS SCRIPTS **/
const getScripts = async (params?: ScriptsRequestParams) => {
  runnerScripts.value = await getListScripts(params);
};
const handleSaveScriptNew = async (params: SaveScriptParams) => {
  await Promise.all([
    await createScript(params),
    await getScripts()
  ]);
};
const handleSaveEditedScript = async (params: SaveScriptParams) => {
  const script = await updateScript(params);
  runnerScripts.value = runnerScripts.value
      .map((s) => s.name === script.name ? { ...s, ...script } : s);
};
const handleSaveScriptWorkedResult = async (params: SaveScriptParams) => {
  const { name, result } = params;

  await updateScript({ name, result });
}
const handleSearchScript = async (query: string) => {
  isSearchScripts.value = true;
  await getScripts({ ...(query && { query }) });
  isSearchScripts.value = false;
};
const handleDeleteScript = async (name: string) => {
  await Promise.all([
    await deleteScript(name),
    await getScripts(),
  ])
};

/** RUNS SCRIPTS **/
const handleRunSingleScript = async (name: string) => {
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

  const response = await runSingleScript(name);

  runnerScripts.value = runnerScripts.value
      .map((script) => script.name === name ? {
            ...script,
            statusRun: {
              isCompleted: !response.stderr,
              variant: !response.stderr ? 'default' : 'destructive',
              loading: false,
            },
            result: response.stdout || response.stderr || '',
          }
          : script);
}
const handleRunScripts = async () => {
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

  const response = await runScripts(nameSelectScripts.value);

  runnerScripts.value = runnerScripts.value
      .map((script) => nameSelectScripts.value.includes(script.name)
          ? {
            ...script,
            statusRun: {
              isCompleted: !response.results[script.name]?.stderr,
              variant: !response.results[script.name]?.stderr ? 'default' : 'destructive',
              loading: false,
            },
            result: response.results[script.name]?.stdout ||
                response.results[script.name]?.stderr || '',
          }
          : script);
}

onMounted(getScripts);
</script>

<template>
  <PanelRunAndEditedScripts
      v-model:isSearchScripts="isSearchScripts"
      v-model:runnerScripts="runnerScripts"
      :username="user || 'No name'"
      @onLogout="logout"
      @onSaveScriptNew="handleSaveScriptNew"
      @onSaveEditedScript="handleSaveEditedScript"
      @onDeleteScript="handleDeleteScript"
      @onRunSingleScript="handleRunSingleScript"
      @onRunScripts="handleRunScripts"
      @onSearchScript="handleSearchScript"
      @onSaveScriptWorkedResult="handleSaveScriptWorkedResult"
      @onSort="getScripts"
  />
</template>

<style scoped></style>
