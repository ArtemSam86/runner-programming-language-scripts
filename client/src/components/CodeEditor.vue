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
import { Codemirror } from 'vue-codemirror';
import { vscodeDark } from '@uiw/codemirror-theme-vscode';
import {computed, ref} from 'vue';
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

import { Badge, type BadgeVariants } from '@/components/ui/badge';
import { Switch } from '@/components/ui/switch';
import ScriptNew from "@/components/ScriptNew.vue";
import ScriptEdit from "@/components/ScriptEdit.vue";
import ScriptNewFromTemplate from "@/components/ScriptNewFromTemplate.vue";
import ScriptView from "@/components/ScriptView.vue";
import ScriptWorkedResult from "@/components/ScriptWorkedResult.vue";
import {Spinner} from "@/components/ui/spinner";

export interface CodeEditorData {
  name: string;
  code: string;
}
interface StatusRun {
  isCompleted: boolean;
  loading?: boolean;
  variant: BadgeVariants['variant'];
}
export interface Script {
  created: string;
  modified: string;
  description: string | null;
  name: string;
  size: number;
}
export interface RunnerScript extends Script {
  isSelected: boolean;
  name: string;
  description: string;
  statusRun: StatusRun;
  scriptRunResult: string;
}

interface Emits {
  createNewScript: [CodeEditorData];
  createNewScriptFromTemplate: [CodeEditorData];
  updateCurrentScript: [CodeEditorData];
  updateDescription: [name: string];
  deleteCurrentScript: [name: string];
  runCurrentScript: [name: string];
  runSelectedScripts: [];
  downloadScriptCode: [string];
  onSearchScript: [value: string];
}

const emits = defineEmits<Emits>();
const { defaultCode = '' } = defineProps<{ defaultCode?: string }>();

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
const editorCode = defineModel<string>('editorCode', { default: '' });

const selectOption = ref('');
const fileName = ref('');

const isRunScripts = computed<boolean>(() =>
    runnerScripts.value.some((script) => script.isSelected && script.statusRun.loading));
const disableButtonRunScripts = computed(() =>
    !runnerScripts.value.some((script) => script.isSelected));


// + TODO: 1. Сброс кода из редактора при закрытии окна
// + TODO: 2. Панель вверху аккуратную с кнопками, или оставить, или кнопки перенести, или сайд-бар
// TODO: 3. Авторизация на бэке
// + TODO: 4. Описание скрипта (добавить/изменить/удалить)
// + TODO: 5. Disable кнопки сохранить при редактировании
// + TODO: 6. Добавить дату создания/обновления скрипта
// TODO: 7. Докер ??? подумать
// TODO: 8. Настроить притер и линтер для клиента
// + TODO: 9. Поправить кнопки Сохранить, Отмена(сделать меньше по ширине)
// ??? TODO: 10. Подумать нужны ли окна для скриптов для ввода аргументов запуска
// ??? TODO: 11. Подумать надо ли окна Drawer разместить
//     TODO: слева(просмотр скрипта)/справа(просмотр результата)/снизу(редакт., созд.)
// + TODO: 12. Сортировка(только при старте на бэке) и поиск(СДЕЛАН)
// TODO:     на фронте emits('onSearchScript', searchValue.value); доработать searchValue.value для разных типов
// TODO: 13. ПРИЧЕСАТЬ КОД. Возможно Pinia(под ??? нужна ли тут) или useStorage из VueUse

const createNewScript = (name: string) => {
  emits('createNewScript', {
    name,
    code: editorCode.value,
  });
};
const createNewScriptFromTemplate = (name: string) => {
  emits('createNewScriptFromTemplate', {
    name,
    code: editorCode.value,
  });
};
const updateCurrentScript = (name: string, typeUpdated: 'desc' | 'code') => {
  if (typeUpdated === 'code') {
    emits('updateCurrentScript', {
      name,
      code: editorCode.value,
    });
  }

  if (typeUpdated === 'desc') {
    emits('updateDescription', name);
  }
};
const deleteCurrentScript = (name: string) => {
  emits('deleteCurrentScript', name);
};
const runCurrentScript = (name: string) => {
  emits('runCurrentScript', name);
};
const runSelectedScripts = () => {
  emits('runSelectedScripts');
};
const downloadScriptCode = (name: string) => {
  selectOption.value = name;
  fileName.value = name.split('.')[0] || '';
  emits('downloadScriptCode', name);
};
const closeScript = () => {
  editorCode.value = defaultCode;
};
</script>

<template>
  <div class="code-editor">
    <!--NEW-->
    <ScriptNew
        :isRun="isRunScripts"
        :isSearch="isSearchScripts"
        :disableButtonRunScripts
        @createNewScript="createNewScript"
        @runSelectedScripts="runSelectedScripts"
        @close="closeScript"
        @onSearchScript="emits('onSearchScript', $event)"
    >
      <template #editorCode>
        <Codemirror
            v-model="editorCode"
            :extensions="defaultExtensions"
        />
      </template>
    </ScriptNew>

    <!--TABLE SCRIPTS-->
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead class="text-center">Выбрать</TableHead>
          <TableHead class="w-0.5">Вид</TableHead>
          <TableHead class="text-left w-1/5">Имя скрипта</TableHead>
          <TableHead class="text-left w-0.5">Дата создания</TableHead>
          <TableHead class="text-left w-0.5">Дата изменения</TableHead>
          <TableHead class="text-left w-0.5">Размер</TableHead>
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
                @downloadScriptCode="downloadScriptCode"
                @close="closeScript"
            >
              <template #editorCode>
                <Codemirror
                    v-model="editorCode"
                    :extensions="defaultExtensions"
                />
              </template>
            </ScriptView>
          </TableCell>
          <TableCell class="text-left w-fit">{{ script.name }}</TableCell>
          <TableCell class="text-left w-fit">{{ script.created }}</TableCell>
          <TableCell class="text-left w-fit">{{ script.modified }}</TableCell>
          <TableCell class="text-left w-fit">{{ script.size }} Байт</TableCell>
          <TableCell class="text-left">{{script.description}}</TableCell>
          <TableCell class="text-center">
            <Badge
                class="transition-all duration-400"
                :variant="script.statusRun.variant"
            >
              {{script.statusRun.isCompleted ? 'Выполнен' : 'Не выполнен'}}
            </Badge>
          </TableCell>
          <TableCell class="text-center">
            <!--RESULT-->
            <ScriptWorkedResult :scriptName="script.name">
              <template #editorCode>
                <Codemirror
                    v-model="script.scriptRunResult"
                    :extensions="defaultExtensions"
                />
              </template>
            </ScriptWorkedResult>
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
                <DropdownMenuItem @click="runCurrentScript(script.name)">
                  <Play />
                  Запустить скрипт
                </DropdownMenuItem>
                <!--Modal-->
                <!-- isOpenDrawerEdit -->
                <DropdownMenuItem>
                  <ScriptEdit
                      :scriptName="script.name"
                      @downloadScriptCode="downloadScriptCode"
                      @updateCurrentScript="updateCurrentScript"
                      @close="closeScript"
                  >
                    <template #editorCode>
                      <Codemirror
                          v-model="editorCode"
                          :extensions="defaultExtensions"
                      />
                    </template>
                  </ScriptEdit>
                </DropdownMenuItem>
                <!-- isOpenDrawerNewFromCurrent -->
                <DropdownMenuItem>
                  <ScriptNewFromTemplate
                      :scriptName="script.name"
                      @downloadScriptCode="downloadScriptCode"
                      @createNewScriptFromCurrent="createNewScriptFromTemplate"
                      @close="closeScript"
                  >
                    <template #editorCode>
                      <Codemirror
                          v-model="editorCode"
                          :extensions="defaultExtensions"
                      />
                    </template>
                  </ScriptNewFromTemplate>
                </DropdownMenuItem>
                <DropdownMenuItem @click="deleteCurrentScript(script.name)">
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

<style scoped lang="scss">
.code-editor {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px;
}
</style>
