<script setup lang="ts">
import {
  Drawer, DrawerClose,
  DrawerContent,
  DrawerDescription, DrawerFooter,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger
} from "@/components/ui/drawer";
import {computed, ref} from "vue";
import {FileText, File} from "lucide-vue-next";
import {Button} from "@/components/ui/button";
import {Codemirror} from "vue-codemirror";
import type {Extension} from "@codemirror/state";
import type {SaveScriptParams} from "@/shared/types/components.ts";
import ScriptCopy from "@/components/ScriptCopy.vue";

interface Emits {
  onSaveScriptWorkedResult: [params: SaveScriptParams];
  close: [];
}

const emits = defineEmits<Emits>();

const { scriptName } = defineProps<{ scriptName: string; extensions: Extension[] }>();

const isOpen = ref(false);
const result = defineModel('result', { default: '' });

const jsonParseStringify = (value: string) => {
  try {
    return JSON.stringify(JSON.parse(value), null, 2);
  } catch {
    return value;
  }
};

const buttonDisabled = computed(() => !result.value.length);
const resultValue = computed({
  get: () => jsonParseStringify(result.value),
  set: (newValue: string) => {
    result.value = newValue;
  }
});

const onSave = () => {
  emits('onSaveScriptWorkedResult', {
    name: scriptName,
    result: result.value,
  });
  isOpen.value = false;
};
const onClear = () => {
  resultValue.value = '';
};
</script>

<template>
  <Drawer v-model:open="isOpen">
    <DrawerTrigger>
      <FileText
          v-if="resultValue"
          class="size-5 cursor-pointer hover:stroke-primary transition-all duration-300"
      />
      <File
          v-else
          class="size-5 cursor-pointer hover:stroke-primary transition-all duration-300"
      />
    </DrawerTrigger>
    <DrawerContent>
      <div class="mx-auto w-full max-w-full">
        <DrawerHeader class="flex flex-row items-center gap-2">
          <DrawerTitle>
            {{scriptName}}
          </DrawerTitle>
          <DrawerDescription>
            <ScriptCopy
                v-if="resultValue"
                :copyText="resultValue"
            />
          </DrawerDescription>
        </DrawerHeader>

        <div class="pl-4 pr-4">
          <Codemirror
              v-model="resultValue"
              :extensions
          />
        </div>

        <DrawerFooter class="flex flex-row items-center gap-2">
          <Button
              class="cursor-pointer"
              @click="onSave"
          >
            Сохранить
          </Button>
          <Button
              class="cursor-pointer"
              variant="outline"
              :disabled="buttonDisabled"
              @click="onClear"
          >
            Очистить
          </Button>
          <DrawerClose asChild>
            <Button
                class="cursor-pointer"
                variant="outline"
            >
              Закрыть
            </Button>
          </DrawerClose>
        </DrawerFooter>

      </div>
    </DrawerContent>
  </Drawer>
</template>

<style scoped>

</style>