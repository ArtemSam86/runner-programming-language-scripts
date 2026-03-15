<script setup lang="ts">
import { EditorState, type Extension } from '@codemirror/state';
import {
  keymap,
  highlightSpecialChars,
  drawSelection,
  highlightActiveLine,
  dropCursor,
  rectangularSelection,
  crosshairCursor,
  lineNumbers,
  highlightActiveLineGutter,
  EditorView,
} from '@codemirror/view';
import {
  indentOnInput,
  bracketMatching,
  foldGutter,
  foldKeymap,
} from '@codemirror/language';
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
import { searchKeymap, highlightSelectionMatches } from '@codemirror/search';
import {
  autocompletion,
  completionKeymap,
  closeBrackets,
  closeBracketsKeymap,
} from '@codemirror/autocomplete';
import { lintKeymap } from '@codemirror/lint';
import { python } from '@codemirror/lang-python';
import {json} from '@codemirror/lang-json';
import { vscodeDark } from '@uiw/codemirror-theme-vscode';
import {computed} from 'vue';
import {Menu, Play, Trash2} from "lucide-vue-next";

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';

import { Switch } from '@/components/ui/switch';
import ScriptNew from "@/components/ScriptNew.vue";
import ScriptEdit from "@/components/ScriptEdit.vue";
import ScriptNewFrom from "@/components/ScriptNewFrom.vue";
import ScriptView from "@/components/ScriptView.vue";
import ScriptWorkedResult from "@/components/ScriptWorkedResult.vue";
import {Spinner} from "@/components/ui/spinner";
import type {RunnerScript} from "@/shared/types/common.ts";
import {Button} from "@/components/ui/button";
import ScriptSearch from "@/components/ScriptSearch.vue";
import {DEFAULT_CODE} from "@/shared/constants.ts";
import type {SaveScriptParams} from "@/shared/types/components.ts";
import ScriptSort from "@/components/ScriptSort.vue";
import type {ScriptsRequestParams} from "@/entities/scripts/types.ts";
import ScriptStatus from "@/components/ScriptStatus.vue";
import ScriptMenu from "@/components/ScriptMenu.vue";

interface Emits {
  onSaveScriptNew: [params: SaveScriptParams];
  onSaveEditedScript: [params: SaveScriptParams];
  onSaveScriptWorkedResult: [params: SaveScriptParams];
  onDeleteScript: [name: string];
  onRunSingleScript: [name: string];
  onRunScripts: [];
  onSearchScript: [value: string];
  onSort: [params: ScriptsRequestParams];
  onLogout: [];
}

const emits = defineEmits<Emits>();
defineProps<{ username: string }>();

const defaultExtensions: Extension[] = [
  vscodeDark,
  python(),
  json(),
  // A line number gutter
  lineNumbers(),
  // A gutter with code folding markers
  foldGutter(),
  // Replace non-printable characters with placeholders
  highlightSpecialChars(),
  // The undo history
  history(),
  // Replace native cursor/selection with our own
  drawSelection(),
  // Show a drop cursor when dragging over the editor
  dropCursor(),
  // Allow multiple cursors/selections
  EditorState.allowMultipleSelections.of(true),
  EditorView.theme({ ".cm-scroller": {height: "50vh"} }),
  // Re-indent lines when typing specific input
  indentOnInput(),
  // Highlight syntax with a default style
  // syntaxHighlighting(defaultHighlightStyle),
  // Highlight matching brackets near cursor
  bracketMatching(),
  // Automatically close brackets
  closeBrackets(),
  // Load the autocompletion system
  autocompletion(),
  // Allow alt-drag to select rectangular regions
  rectangularSelection(),
  // Change the cursor to a crosshair when holding alt
  crosshairCursor(),
  // Style the current line specially
  highlightActiveLine(),
  // Style the gutter for current line specially
  highlightActiveLineGutter(),
  // Highlight text that matches the selected text
  highlightSelectionMatches(),
  keymap.of([
    // Closed-brackets aware backspace
    ...closeBracketsKeymap,
    // A large set of basic bindings
    ...defaultKeymap,
    // Search-related keys
    ...searchKeymap,
    // Redo/undo keys
    ...historyKeymap,
    // Code folding bindings
    ...foldKeymap,
    // Autocompletion keys
    ...completionKeymap,
    // Keys related to the linter system
    ...lintKeymap,
  ]),
];

const runnerScripts = defineModel<RunnerScript[]>('runnerScripts', { default: [] });
const isSearchScripts = defineModel<boolean>('isSearchScripts', { default: false });

const isRunScripts = computed<boolean>(() =>
    runnerScripts.value.some((script) => script.isSelected && script.statusRun.loading));
const disableButtonRunScripts = computed(() =>
    !runnerScripts.value.some((script) => script.isSelected));

// + TODO: 0. Запуск скриптов проверить/поправить
// + TODO: 1. Авторизация на бэке
// + TODO: 2. Сортировка
// ??? TODO: 3. Аргументы командной строки для запуска скрипта
// TODO: -----------
// - TODO: 4. Настроить притер и линтер для клиента
// TODO: 5. Докер ??? подумать
// ??? TODO: 6. Заметки <Scroll /> <ScrollText />
// + TODO: 7. Копирование кода(+)/результата(+) в буфер обмена <Copy />
</script>

<template>
  <div class="flex flex-col gap-4 p-4">
    <div class="flex items-center gap-2">
      <!--MENU-->
      <ScriptMenu
          :username="username"
          @onLogout="emits('onLogout')"
      />
      <!--NEW-->
      <ScriptNew
          :defaultCode="DEFAULT_CODE"
          :extensions="defaultExtensions"
          @onSaveScriptNew="emits('onSaveScriptNew', $event)"
      />
      <Button
          class="cursor-pointer"
          :disabled="disableButtonRunScripts || isRunScripts"
          @click="emits('onRunScripts')"
      >
        <Spinner v-if="isRunScripts" class="size-5" />
        <Play v-else class="size-5" />
        Запустить скрипты
      </Button>
      <ScriptSearch
          :isSearch="isSearchScripts"
          @onSearchScript="emits('onSearchScript', $event)"
      />
    </div>

    <!--TABLE SCRIPTS-->
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead class="text-center">Выбрать</TableHead>
          <TableHead class="w-0.5">Вид</TableHead>
          <TableHead class="text-left w-1/5">
            <ScriptSort
                type="name"
                label="Имя скрипта"
                @onSort="emits('onSort', $event)"
            />
          </TableHead>
          <TableHead class="text-left w-0.5">
            <ScriptSort
                type="created"
                label="Дата создания"
                @onSort="emits('onSort', $event)"
            />
          </TableHead>
          <TableHead class="text-left w-0.5">
            <ScriptSort
                type="modified"
                label="Дата изменения"
                @onSort="emits('onSort', $event)"
            />
          </TableHead>
          <TableHead class="text-left w-0.5">
            <ScriptSort
                type="size"
                label="Размер"
                @onSort="emits('onSort', $event)"
            />
          </TableHead>
          <TableHead class="text-left w-fit">Описание</TableHead>
          <TableHead class="text-center w-32">Статус</TableHead>
          <TableHead class="w-0.5">Результат</TableHead>
          <TableHead class="text-center w-0.5">Действия</TableHead>
        </TableRow>
      </TableHeader>

      <TableBody>
        <TableRow
            v-for="script in runnerScripts"
            :key="script.name"
            class="hover:bg-yellow-100"
        >
          <TableCell class="w-0.5">
            <Switch
                :disabled="script.statusRun.loading"
                v-model="script.isSelected"
                class="cursor-pointer"
            />
          </TableCell>
          <TableCell class="w-0.5">
            <ScriptView
                :scriptName="script.name"
                :code="script.code"
                :description="script.description"
                :extensions="defaultExtensions"
            />
          </TableCell>
          <TableCell class="text-left w-fit">{{ script.name }}</TableCell>
          <TableCell class="text-left w-fit">{{ script.created }}</TableCell>
          <TableCell class="text-left w-fit">{{ script.modified }}</TableCell>
          <TableCell class="text-left w-fit">{{ script.size }} Байт</TableCell>
          <TableCell class="text-left">{{script.description}}</TableCell>
          <TableCell class="text-center">
            <ScriptStatus :variant="script.statusRun.variant" />
          </TableCell>
          <TableCell class="text-center">
            <!--RESULT-->
            <ScriptWorkedResult
                v-model:result="script.result"
                :scriptName="script.name"
                :extensions="defaultExtensions"
                @onSaveScriptWorkedResult="emits('onSaveScriptWorkedResult', $event)"
            />
          </TableCell>
          <TableCell class="text-center">
            <DropdownMenu>
              <DropdownMenuTrigger :disabled="script.statusRun.loading">
                <Spinner
                    v-if="script.statusRun.loading"
                    class="size-5"
                />
                <Menu
                    v-else
                    class="size-5 cursor-pointer hover:stroke-primary transition-all duration-300"
                />
              </DropdownMenuTrigger>
              <DropdownMenuContent>
                <DropdownMenuLabel>Действия</DropdownMenuLabel>
                <DropdownMenuSeparator />
                <!-- ACTIONS -->
                <DropdownMenuItem @click="emits('onRunSingleScript', script.name)">
                  <Play />
                  Запустить скрипт
                </DropdownMenuItem>
                <!-- Modal Edit -->
                <DropdownMenuItem>
                  <ScriptEdit
                      :scriptName="script.name"
                      :scriptCode="script.code"
                      :scriptDescription="script.description"
                      :extensions="defaultExtensions"
                      @onSaveEditedScript="emits('onSaveEditedScript', $event)"
                  />
                </DropdownMenuItem>
                <!-- Modal New From Current -->
                <DropdownMenuItem>
                  <ScriptNewFrom
                      :extensions="defaultExtensions"
                      :name="script.name"
                      :code="script.code"
                      :description="script.description"
                      @onSaveScriptNewFrom="emits('onSaveScriptNew', $event)"
                  />
                </DropdownMenuItem>
                <DropdownMenuItem
                    variant="destructive"
                    @click="emits('onDeleteScript', script.name)"
                >
                  <Trash2 />
                  Удалить скрипт
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </TableCell>
        </TableRow>
      </TableBody>
    </Table>
  </div>
</template>

<style scoped></style>
