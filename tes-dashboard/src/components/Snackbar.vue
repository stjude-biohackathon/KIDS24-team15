<template>
  <!-- Div containing the Snackbar element used for error notification -->
  <div class="text-center">
    <v-snackbar v-model="internalSnack" :timeout="timeout" :color="snackColor" top>
      {{ snackText }}
      <template v-slot:actions>
        <v-btn text="Close" @click="emit('update:modelValue', false)"/>
      </template>
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, watchEffect } from 'vue'

const emit = defineEmits(['update:modelValue'])
const props = defineProps({
  snack: Boolean,
  snackColor: String,
  snackText: String
})

const internalSnack = ref(props.snack)
const timeout = ref(-1);

watchEffect(() => {
  internalSnack.value = props.snack;
})
</script>
