<script setup lang="ts">
import {
  CalendarArrowDown,
  CalendarArrowUp,
  ArrowDownAZ,
  ArrowDownZA,
  ArrowUp01,
  ArrowUp10
} from "lucide-vue-next";
import {computed, type FunctionalComponent, ref} from "vue";
import type {SortBy, SortOrder} from "@/shared/types/common.ts";
import type {ScriptsRequestParams} from "@/entities/scripts/types.ts";

interface Emits {
  onSort: [params: ScriptsRequestParams]
}

interface Props {
  type: SortBy;
  label?: string;
}

const { type } = defineProps<Props>();
const emits = defineEmits<Emits>();

const components: Record<SortBy, Record<SortOrder, FunctionalComponent>> = {
  created: { asc: CalendarArrowUp, desc: CalendarArrowDown },
  modified: { asc: CalendarArrowUp, desc: CalendarArrowDown },
  name: { asc: ArrowDownAZ, desc: ArrowDownZA },
  size: { asc: ArrowUp01, desc: ArrowUp10 },
}
const orders: Record<SortOrder, SortOrder> = {
  asc: 'desc',
  desc: 'asc'
}

const sortOrder = ref<SortOrder>('desc');

const component = computed(() => components[type][sortOrder.value]);

const onSort = () => {
  sortOrder.value = orders[sortOrder.value];
  emits("onSort", { sort_by: type, sort_order: sortOrder.value });
};
</script>

<template>
  <div
      class="group flex items-center gap-2 cursor-pointer"
      @click="onSort"
  >
    <span
        class="select-none group-hover:text-primary transition-all duration-300"
        v-if="label"
    >
      {{label}}
    </span>
    <component
        class="size-5 group-hover:stroke-primary transition-all duration-300"
        :is="component"
    />
  </div>
</template>

<style scoped></style>