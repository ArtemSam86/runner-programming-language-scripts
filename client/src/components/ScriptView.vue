<script setup lang="ts">
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle, DrawerTrigger
} from "@/components/ui/drawer";
import {FileCode, File} from "lucide-vue-next";
import {Codemirror} from "vue-codemirror";
import type {Extension} from "@codemirror/state";
import ScriptCopy from "@/components/ScriptCopy.vue";

interface Props {
  scriptName: string;
  code: string;
  description?: string;
  extensions: Extension[];
}

const { scriptName, description = '' } = defineProps<Props>();
</script>

<template>
  <Drawer>
    <DrawerTrigger @click.stop>
      <FileCode
          v-if="code"
          class="size-5 cursor-pointer hover:stroke-primary transition-all duration-300"
      />
      <File
          v-else
          class="size-5 cursor-pointer hover:stroke-primary transition-all duration-300"
      />
    </DrawerTrigger>
    <DrawerContent>
      <div class="mx-auto w-full max-w-full">
        <DrawerHeader>
          <DrawerTitle class="flex items-center gap-2">
            {{scriptName}}
            <ScriptCopy
                v-if="code"
                :copyText="code"
            />
          </DrawerTitle>
          <DrawerDescription>
            {{ description }}
          </DrawerDescription>
        </DrawerHeader>

        <div class="pl-4 pr-4 pb-4">
          <Codemirror
              :modelValue="code"
              :extensions
          />
        </div>
      </div>
    </DrawerContent>
  </Drawer>
</template>

<style scoped>

</style>