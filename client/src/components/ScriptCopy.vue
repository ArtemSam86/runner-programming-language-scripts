<script setup lang="ts">
import {Copy, CopyCheck} from "lucide-vue-next";
import {ref} from "vue";

const { copyText } = defineProps<{ copyText: string }>();

const checked = ref(false);

const onCopy = async () => {
  await window.navigator.clipboard.writeText(copyText);
  checked.value = !checked.value;
  setTimeout(() => {
    checked.value = !checked.value;
  }, 3000);
};
</script>

<template>
  <div class="flex items-center gap-2">
    <CopyCheck
        v-if="checked"
        class="size-4"
    />
    <Copy
        v-else
        class="size-4 cursor-pointer hover:stroke-primary transition-all duration-300"
        @click="onCopy"
    />
  </div>
</template>

<style scoped>

</style>