<script setup lang="ts">
import {Badge} from "@/components/ui/badge";
import {computed} from "vue";
import type {Statuses, Variant} from "@/shared/types/components.ts";

interface Props {
  variant: Variant;
}

defineProps<Props>();

const statuses = computed<Statuses>(() => ({
  default: 'SUCCESS',
  destructive: 'ERROR',
  secondary: 'NEUTRAL',
}));

</script>

<template>
  <Badge
      class="transition-all duration-400"
      :variant="variant"
  >
    {{ statuses[variant] }}
  </Badge>
</template>

<style scoped>

</style>