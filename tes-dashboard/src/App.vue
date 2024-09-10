<template>
  <v-app>
    <AppBar />
    <v-main>
      <router-view />
    </v-main>
  </v-app>
</template>

<script setup lang="ts">
import ApiService from '@/services/api.service.ts'
import AppBar from '@/components/AppBar.vue'
import { onMounted } from 'vue'
import { useAppStore } from '@/store/app.ts'
import { useRoute } from 'vue-router'


const store = useAppStore()
const route = useRoute()

onMounted(() => {
  ApiService.setHeader()
})
</script>
