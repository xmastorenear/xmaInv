<script setup lang="ts">
import { ref } from 'vue';

// Теперь передаем объект с именем и цветом
const emit = defineEmits<{
  (e: 'create', payload: { name: string, color: string }): void
}>();

const inputName = ref('');
// Список доступных цветов для выбора
const colors = ['#00c087', '#ff5f56', '#ffbd2e', '#4facfe', '#a166ab', '#f87171'];
const selectedColor = ref(colors[0]);

function submit() {
  if (inputName.value.trim()) {
    emit('create', {
      name: inputName.value.trim(),
      color: selectedColor.value
    });
    inputName.value = '';
  }
}
</script>

<template>
  <div class="modal-card">
    <h2>Новая стратегия</h2>
    <p class="subtitle">Введите название и выберите цвет</p>

    <input
        v-model="inputName"
        placeholder="Например: Скальпинг BTC"
        @keyup.enter="submit"
        autofocus
    >

    <!-- Выбор цвета -->
    <div class="color-picker">
      <div
        v-for="color in colors"
        :key="color"
        class="color-option"
        :style="{ backgroundColor: color }"
        :class="{ active: selectedColor === color }"
        @click="selectedColor = color"
      ></div>
    </div>

    <button @click="submit" :disabled="!inputName.trim()">
      Создать стратегию
    </button>
  </div>
</template>

<style scoped>
.modal-card {
  background: #1a1d21;
  padding: 2rem;
  border-radius: 16px;
  border: 1px solid #2d333b;
  width: 100%;
  max-width: 360px;
  text-align: center;
}

.color-picker {
  display: flex;
  justify-content: center;
  gap: 10px;
  margin-bottom: 1.5rem;
}

.color-option {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  cursor: pointer;
  border: 2px solid transparent;
  transition: transform 0.2s;
}

.color-option.active {
  border-color: white;
  transform: scale(1.2);
}

input {
  width: 100%;
  box-sizing: border-box;
  padding: 0.8rem;
  background: #0f1113;
  border: 1px solid #2d333b;
  border-radius: 10px;
  color: white;
  margin-bottom: 1rem;
}

button {
  width: 100%;
  padding: 0.8rem;
  background: #00c087;
  color: #000;
  border: none;
  border-radius: 10px;
  font-weight: 700;
  cursor: pointer;
}
/* ... остальные ваши стили ... */
</style>
