<script setup lang="ts">
import {
  Drawer,
  DrawerClose,
  DrawerContent,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger
} from "@/components/ui/drawer";
import {Button} from "@/components/ui/button";
import {Pencil} from "lucide-vue-next";
import {ref} from "vue";
import {Input} from "@/components/ui/input";
import {Codemirror} from "vue-codemirror";
import type {Extension} from "@codemirror/state";
import type {SaveScriptParams} from "@/shared/types/components.ts";

interface Emits {
  onSaveEditedScript: [params: SaveScriptParams];
}
interface Props {
  scriptName: string;
  scriptCode: string;
  scriptDescription: string;
  extensions: Extension[];
}

const emits = defineEmits<Emits>();
const { scriptName, scriptCode, scriptDescription } = defineProps<Props>();

const code = ref(scriptCode);
const description = ref(scriptDescription);
const isOpen = ref(false);

const onSave = () => {
  emits('onSaveEditedScript', {
    name: scriptName,
    code: code.value,
    description: description.value,
  });
  isOpen.value = false;
};
const handleClose = () => {
  code.value = scriptCode;
  description.value = scriptDescription;
};
</script>

<template>
  <Drawer v-model:open="isOpen">
    <DrawerTrigger
        class="flex gap-2 items-center"
        @click.stop
    >
      <Pencil />
      Редактировать скрипт
    </DrawerTrigger>
    <DrawerContent>
      <div class="mx-auto w-full max-w-full">
        <DrawerHeader>
          <DrawerTitle class="flex items-center gap-2">
            <p>Имя скрипта:</p>
            {{scriptName}}
          </DrawerTitle>
          <DrawerDescription class="flex items-center gap-2">
            Описание:
            <Input v-model="description" />
          </DrawerDescription>
        </DrawerHeader>

        <div class="pl-4 pr-4">
          <Codemirror
              v-model="code"
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
          <DrawerClose asChild>
            <Button
                class="cursor-pointer"
                variant="outline"
                @click="handleClose"
            >
              Отмена
            </Button>
          </DrawerClose>
        </DrawerFooter>
      </div>
    </DrawerContent>
  </Drawer>
</template>

<style scoped>

</style>