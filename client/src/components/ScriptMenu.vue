<script setup lang="ts">
import {Menu, LogOut, Settings, User} from 'lucide-vue-next';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';

const emits = defineEmits<{ onLogout: [] }>();
defineProps<{ username: string; }>();
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button class="cursor-pointer">
        <Menu class="size-5" />
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent>
      <DropdownMenuLabel class="flex items-center gap-2">
        <User class="size-4" />
        {{username}}
      </DropdownMenuLabel>

      <DropdownMenuSeparator />

      <DropdownMenuItem class="flex items-center gap-2 cursor-pointer">
        <Settings />
        Настройки
      </DropdownMenuItem>

      <DropdownMenuSeparator />

      <DropdownMenuItem
          class="flex items-center gap-2 cursor-pointer"
          variant="destructive"
          @click="emits('onLogout')"
      >
        <LogOut />
        Выйти
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>