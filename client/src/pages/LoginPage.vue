<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useForm } from 'vee-validate';
import { toTypedSchema } from '@vee-validate/zod';
import * as z from 'zod';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { useAuth } from '@/composables/useAuth';
import {Spinner} from "@/components/ui/spinner";
import {PANEL_NAME, REGISTER_NAME} from "@/router/constants.ts";

const router = useRouter();
const { login } = useAuth();
const loginError = ref<string | null>(null);
const isSubmitting = ref(false);

const formSchema = toTypedSchema(
    z.object({
      username: z
          .string()
          .min(1, 'Имя пользователя обязательно')
          .min(2, 'Минимум 2 символа')
          .max(50, 'Максимум 50 символов'),
      password: z
          .string()
          .min(1, 'Пароль обязателен')
          .min(6, 'Минимум 6 символов')
          .max(32, 'Максимум 32 символа'),
    })
);

const { handleSubmit, isFieldDirty } = useForm({
  validationSchema: formSchema,
  initialValues: {
    username: '',
    password: '',
  },
});

const onSubmit = handleSubmit(async (values) => {
  loginError.value = null;
  isSubmitting.value = true;

  try {
    await login(values);
    router.push({ name: PANEL_NAME });
  } catch (error: any) {
    console.error('Login error', error);
    if (error.response) {
      const status = error.response.status;
      if (status === 401) {
        loginError.value = 'Неверное имя пользователя или пароль';
      } else if (status === 404) {
        loginError.value = 'Пользователь не найден. Переход к регистрации...';
        setTimeout(() => {
          router.push({ name: REGISTER_NAME });
        }, 1500);
      } else {
        loginError.value = 'Произошла ошибка. Попробуйте позже.';
      }
    } else {
      loginError.value = 'Сетевая ошибка. Проверьте подключение.';
    }
  } finally {
    isSubmitting.value = false;
  }
});
</script>

<template>
  <div class="flex items-center justify-center h-[100vh]">
    <div class="w-1/3 p-6 bg-white rounded shadow">
      <h1 class="text-2xl font-bold mb-6">Вход</h1>
      <form @submit="onSubmit" class="space-y-4">
        <FormField v-slot="{ componentField }" name="username" :validate-on-blur="!isFieldDirty">
          <FormItem>
            <FormLabel>Имя пользователя</FormLabel>
            <FormControl>
              <Input type="text" placeholder="Введите имя" v-bind="componentField" />
            </FormControl>
            <FormMessage />
          </FormItem>
        </FormField>

        <FormField v-slot="{ componentField }" name="password" :validate-on-blur="!isFieldDirty">
          <FormItem>
            <FormLabel>Пароль</FormLabel>
            <FormControl>
              <Input type="password" placeholder="Введите пароль" v-bind="componentField" />
            </FormControl>
            <FormMessage />
          </FormItem>
        </FormField>

        <div v-if="loginError" class="text-sm text-red-600">
          {{ loginError }}
        </div>

        <Button class="cursor-pointer" type="submit" :disabled="isSubmitting">
          {{ isSubmitting ? 'Вход' : 'Войти' }}
          <Spinner
              v-if="isSubmitting"
              class="size-5"
          />
        </Button>

        <div class="text-sm text-center mt-4">
          Нет аккаунта?
          <router-link :to="{ name: REGISTER_NAME }" class="text-blue-600 hover:underline">
            Зарегистрироваться
          </router-link>
        </div>
      </form>
    </div>
  </div>
</template>