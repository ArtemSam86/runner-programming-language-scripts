<script setup lang="ts">
import {
  Drawer,
  DrawerClose,
  DrawerContent,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle, DrawerTrigger
} from "@/components/ui/drawer";
import {Button} from "@/components/ui/button";
import {Input} from "@/components/ui/input";
import {Plus} from "lucide-vue-next";
import {computed, ref} from "vue";

interface Emits {
  downloadScriptCode: [name: string];
  createNewScriptFromCurrent: [name: string];
  close: [];
}

const emits = defineEmits<Emits>();
const { scriptName } = defineProps<{ scriptName: string }>();


const fileName = ref(scriptName.replace('.py', ''));
const isOpenDrawerNewFromCurrent = ref(false);

const fileNameWithExtensions = computed(() => fileName.value + '.py');
const buttonDisabled = computed(() => fileName.value.length < 6);

const createNewScriptFromCurrent = () => {
  emits('createNewScriptFromCurrent', fileNameWithExtensions.value);
  isOpenDrawerNewFromCurrent.value = false;
};
const downloadScriptCode = () => {
  emits('downloadScriptCode', scriptName);
};
</script>

<template>
  <Drawer
      v-model:open="isOpenDrawerNewFromCurrent"
      @close="emits('close')"
  >
    <DrawerTrigger
        class="flex gap-2 items-center"
        @click.stop="downloadScriptCode"
    >
      <Plus />
      Новый скрипт из выбранного
    </DrawerTrigger>
    <DrawerContent>
      <div class="mx-auto w-full max-w-full">
        <DrawerHeader>
          <DrawerTitle class="flex items-center gap-2">
            <p>Имя скрипта:</p>
            <Input
                class="w-fit"
                v-model="fileName"
                type="text"
                placeholder="Введите имя скрипта"
            />
          </DrawerTitle>
          <DrawerDescription>Код скрипта</DrawerDescription>
        </DrawerHeader>

        <div class="pl-4 pr-4">
          <slot name="editorCode" />
        </div>

        <DrawerFooter class="flex flex-row items-center gap-2">
          <Button
              class="cursor-pointer"
              :disabled="buttonDisabled"
              @click="createNewScriptFromCurrent"
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