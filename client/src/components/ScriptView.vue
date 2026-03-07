<script setup lang="ts">
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle, DrawerTrigger
} from "@/components/ui/drawer";
import {ref} from "vue";
import {FileCode} from "lucide-vue-next";


interface Emits {
  downloadScriptCode: [name: string];
  close: [];
}

const emits = defineEmits<Emits>();
const { scriptName } = defineProps<{ scriptName: string }>();

const isOpenDrawerView = ref(false);

const downloadScriptCode = () => {
  emits('downloadScriptCode', scriptName);
};
</script>

<template>
  <Drawer
      v-model:open="isOpenDrawerView"
      @close="emits('close')"
  >
    <DrawerTrigger @click.stop="downloadScriptCode">
      <FileCode class="size-5 cursor-pointer hover:stroke-primary transition-all duration-300" />
    </DrawerTrigger>
    <DrawerContent>
      <div class="mx-auto w-full max-w-full">
        <DrawerHeader>
          <DrawerTitle class="flex items-center gap-2">
            <p>Имя скрипта:</p>
            {{scriptName}}
          </DrawerTitle>
          <DrawerDescription>Код скрипта</DrawerDescription>
        </DrawerHeader>

        <div class="pl-4 pr-4 pb-4">
          <slot name="editorCode" />
        </div>
      </div>
    </DrawerContent>
  </Drawer>
</template>

<style scoped>

</style>