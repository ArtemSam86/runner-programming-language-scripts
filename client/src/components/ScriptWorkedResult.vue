<script setup lang="ts">
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger
} from "@/components/ui/drawer";
import {ref} from "vue";
import {FileText} from "lucide-vue-next";

const { scriptName } = defineProps<{ scriptName: string }>();

const isOpenDrawerResult = ref(false);
</script>

<template>
  <Drawer v-model:open="isOpenDrawerResult">
    <DrawerTrigger>
      <FileText class="size-5 cursor-pointer hover:stroke-primary transition-all duration-300" />
    </DrawerTrigger>
    <DrawerContent>
      <div class="mx-auto w-full max-w-full">
        <DrawerHeader>
          <DrawerTitle class="flex items-center gap-2">
            Имя скрипта: {{scriptName}}
          </DrawerTitle>
          <DrawerDescription>Результат работы:</DrawerDescription>
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