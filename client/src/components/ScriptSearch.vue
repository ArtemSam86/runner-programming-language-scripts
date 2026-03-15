<script setup lang="ts">
import {Search, X} from "lucide-vue-next";
import {Spinner} from "@/components/ui/spinner";
import {Input} from "@/components/ui/input";
import {computed, ref} from "vue";
import { useDebounceFn } from '@vueuse/core';

interface Emits {
  onSearchScript: [searchValue: string];
}

const emits = defineEmits<Emits>();
defineProps<{ isSearch?: boolean }>();

const searchValue = ref('');

const searchValueDebounce = computed({
  get: () => searchValue.value,
  set: useDebounceFn((value: string) => {
    searchValue.value = value;
    emits('onSearchScript', value);
  }, 500)
})


const onClearSearch = () => {
  searchValue.value = '';
  emits('onSearchScript', searchValue.value);
};
</script>

<template>
  <div class="flex items-center gap-2 w-full">
    <div class="relative w-full items-center">
      <span
          v-if="searchValue"
          class="absolute end-0 inset-y-0 flex items-center justify-center px-2"
          @click="onClearSearch"
      >
        <Spinner
            v-if="isSearch"
            class="size-5 text-muted-foreground"
        />
        <X
           v-else
           class="size-5 text-muted-foreground cursor-pointer"
        />
      </span>
      <Input
          v-model="searchValueDebounce"
          id="search"
          type="text"
          placeholder="Поиск..."
          class="pl-10"
      />
      <span class="absolute start-0 inset-y-0 flex items-center justify-center px-2">
        <Search class="size-5 text-muted-foreground" />
      </span>
    </div>
  </div>
</template>

<style scoped>

</style>