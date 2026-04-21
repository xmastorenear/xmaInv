<script setup lang="ts">
    import {ref, onMounted} from "vue";
    import {invoke} from "@tauri-apps/api/core";

    interface Strategy {
      id: number,
      name: string;
    }

    const strategy = ref<Strategy | null>(null);
    const inputName =  ref('');
    const isLoading = ref(true);

    onMounted(async () => {
      strategy.value = await invoke<Strategy | null>('get_current_strategy');
      isLoading.value = false;
    });

    async function handleCreate() {
      if (!inputName.value) return;
      strategy.value = await invoke('create_strategy', {name: inputName.value});
    }
</script>

<template>
    <div v-if="!strategy" class="modal-overlay">
      <div class="modal-card">
        <h2>Введите название стратегии</h2>
        <input v-model="inputName" @keyup.enter="handleCreate">
        <button @click="handleCreate">Создать</button>
      </div>
    </div>
    <div v-else>
      <h1>Работаем со стратегией: {{ strategy.name }}</h1>
    </div>
</template>

<style>
  html, body, #app {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    background-color: #191a1c; /* Черный цвет */
    color: #ffffff;           /* Белый текст для контраста */
  }

</style>