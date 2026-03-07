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
import { Spinner } from "@/components/ui/spinner";
import {Plus, Play, Search, X} from "lucide-vue-next";
import {Input} from "@/components/ui/input";
import {computed, ref} from "vue";

interface Emits {
  createNewScript: [name: string];
  runSelectedScripts: [];
  close: [];
  onSearchScript: [value: string];
}
interface Props {
  disableButtonRunScripts: boolean;
  isRun?: boolean;
  isSearch?: boolean;
}

const emits = defineEmits<Emits>();
defineProps<Props>();

const isOpenDrawerNew = ref(false);
const fileName = ref('');
const searchValue = ref('');

const fileNameWithExtensions = computed(() => fileName.value + '.py');
const buttonDisabled = computed(() => fileName.value.length < 6);

const createNewScript = () => {
  emits('createNewScript', fileNameWithExtensions.value);
  isOpenDrawerNew.value = false;
};
const runSelectedScripts = () => {
  emits('runSelectedScripts');
};
const onSearchScript = () => {
  emits('onSearchScript', searchValue.value); // доработать searchValue.value для разных типов
};
const onClearSearch = () => {
  searchValue.value = '';
};
</script>

<template>
  <div class="flex items-center gap-2">
    <!-- isOpenDrawerNew -->
    <Drawer
        v-model:open="isOpenDrawerNew"
        @close="emits('close')"
    >
      <DrawerTrigger>
        <Button class="cursor-pointer">
          <Plus class="size-5" />
          Новый скрипт
        </Button>
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
                @click="createNewScript"
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
    <Button
        class="cursor-pointer"
        :disabled="disableButtonRunScripts || isRun"
        @click="runSelectedScripts"
    >
      <Spinner v-if="isRun" class="size-5" />
      <Play v-else class="size-5" />
      Запустить скрипты
    </Button>
    <div class="relative w-full items-center">
      <span
          v-if="searchValue"
          class="absolute end-0 inset-y-0 flex items-center justify-center px-2"
          @click="onClearSearch"
      >
        <X class="size-5 text-muted-foreground cursor-pointer" />
      </span>
      <Input
          v-model="searchValue"
          id="search"
          type="text"
          placeholder="Поиск..."
          class="pl-10"
      />
      <span class="absolute start-0 inset-y-0 flex items-center justify-center px-2">
        <Search class="size-5 text-muted-foreground" />
      </span>
    </div>
    <Button
        class="cursor-pointer"
        :disabled="isSearch"
        @click="onSearchScript"
    >
      <Spinner v-if="isSearch" class="size-5" />
      <span v-else>Найти</span>
    </Button>
  </div>
</template>

<style scoped>

</style>