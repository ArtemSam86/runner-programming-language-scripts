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
import {Plus} from "lucide-vue-next";
import {Input} from "@/components/ui/input";
import {computed, onUnmounted, ref} from "vue";
import {Codemirror} from "vue-codemirror";
import type {Extension} from "@codemirror/state";
import type {SaveScriptParams} from "@/shared/types/components.ts";

interface Emits {
  onSaveScriptNew: [params: SaveScriptParams];
}

const emits = defineEmits<Emits>();
const { defaultCode = '' } = defineProps<{ extensions: Extension[]; defaultCode?: string }>();

const name = ref('');
const code = ref(defaultCode);
const description = ref('');
const isOpen = ref(false);

const fileName = computed({
  get: () => name.value.replace('.py', ''),
  set: (value: string) => name.value = value + '.py',
});
const buttonDisabled = computed(() => fileName.value.length < 6);

const onSave = () => {
  emits('onSaveScriptNew', {
    name: name.value,
    code:  code.value,
    description: description.value,
  });
  isOpen.value = false;
};
const handleClose = () => {
  code.value = defaultCode;
};

onUnmounted(handleClose);
</script>

<template>
  <!-- isOpenDrawerNew -->
  <Drawer
      v-model:open="isOpen"
      @close="handleClose"
  >
    <DrawerTrigger :as="'div'">
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
              :disabled="buttonDisabled"
              @click="onSave"
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