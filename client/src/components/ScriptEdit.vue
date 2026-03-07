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

interface Emits {
  downloadScriptCode: [name: string];
  updateCurrentScript: [name: string];
  close: [];
}

const emits = defineEmits<Emits>();
const { scriptName } = defineProps<{ scriptName: string }>();

const description = defineModel('description', { default: '' });

const isOpenDrawerEdit = ref(false);

const updateCurrentScript = () => {
  emits('updateCurrentScript', scriptName);
  isOpenDrawerEdit.value = false;
};
const downloadScriptCode = () => {
  emits('downloadScriptCode', scriptName);
};
</script>

<template>
  <Drawer
      v-model:open="isOpenDrawerEdit"
      @close="emits('close')"
  >
    <DrawerTrigger
        class="flex gap-2 items-center"
        @click.stop="downloadScriptCode"
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
          <slot name="editorCode" />
        </div>

        <DrawerFooter class="flex flex-row items-center gap-2">
          <Button
              class="cursor-pointer"
              @click="updateCurrentScript"
          >
            Сохранить
          </Button>
          <DrawerClose asChild>
            <Button
                class="cursor-pointer"
                variant="outline"
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