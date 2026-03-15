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
import {LOGIN_NAME} from "@/router/constants.ts";

const router = useRouter();
const { register } = useAuth();
const registerError = ref<string | null>(null);
const isSubmitting = ref(false);

const zodSchema = z
    .object({
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
      confirmPassword: z.string().min(1, 'Подтверждение пароля обязательно'),
    })
    .refine((data) => data.password === data.confirmPassword, {
      message: 'Пароли не совпадают',
      path: ['confirmPassword'],
    });

const formSchema = toTypedSchema(zodSchema);

const { handleSubmit, isFieldDirty } = useForm({
  validationSchema: formSchema,
  initialValues: {
    username: '',
    password: '',
    confirmPassword: '',
  },
});

const onSubmit = handleSubmit(async (values) => {
  registerError.value = null;
  isSubmitting.value = true;

  try {
    await register({ username: values.username, password: values.password });
    router.push({ name: LOGIN_NAME });
  } catch (error: any) {
    console.error('Register error', error);
    if (error.response && error.response.status === 409) {
      registerError.value = 'Пользователь с таким именем уже существует';
    } else {
      registerError.value = 'Ошибка при регистрации. Попробуйте позже.';
    }
  } finally {
    isSubmitting.value = false;
  }
});
</script>

<template>
  <div class="flex items-center justify-center h-[100vh]">
    <div class="w-1/3 p-6 bg-white rounded shadow">
      <h1 class="text-2xl font-bold mb-6">Регистрация</h1>
      <form @submit="onSubmit" class="space-y-4">
        <FormField v-slot="{ componentField }" name="username" :validate-on-blur="!isFieldDirty">
          <FormItem>
            <FormLabel>Имя пользователя</FormLabel>
            <FormControl>
              <Input type="text" placeholder="Придумайте имя" v-bind="componentField" />
            </FormControl>
            <FormMessage />
          </FormItem>
        </FormField>

        <FormField v-slot="{ componentField }" name="password" :validate-on-blur="!isFieldDirty">
          <FormItem>
            <FormLabel>Пароль</FormLabel>
            <FormControl>
              <Input type="password" placeholder="Минимум 6 символов" v-bind="componentField" />
            </FormControl>
            <FormMessage />
          </FormItem>
        </FormField>

        <FormField v-slot="{ componentField }" name="confirmPassword" :validate-on-blur="!isFieldDirty">
          <FormItem>
            <FormLabel>Подтверждение пароля</FormLabel>
            <FormControl>
              <Input type="password" placeholder="Повторите пароль" v-bind="componentField" />
            </FormControl>
            <FormMessage />
          </FormItem>
        </FormField>

        <div v-if="registerError" class="text-sm text-red-600">
          {{ registerError }}
        </div>

        <Button class="cursor-pointer" type="submit" :disabled="isSubmitting">
          {{ isSubmitting ? 'Регистрация' : 'Зарегистрироваться' }}
          <Spinner
              v-if="isSubmitting"
              class="size-5"
          />
        </Button>

        <div class="text-sm text-center mt-4">
          Уже есть аккаунт?
          <router-link :to="{ name: LOGIN_NAME }" class="text-blue-600 hover:underline">
            Войти
          </router-link>
        </div>
      </form>
    </div>
  </div>
</template>