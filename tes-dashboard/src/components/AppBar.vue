<template>
  <v-app-bar
      color="grey-darken-4"
      :absolute="true"
      :flat="true"
  >
    <button
        class="ml-8"
        @click="navigateHome"
        :disabled="['login', 'home'].includes(String(route.name))"
    >
      <v-row>
      <img src="@/assets/Sprocket-Logo.svg" :width="35" alt="Sprocket Logo" />
      <v-app-bar-title class="mt-1 ml-5">
        <span><strong>TES Dashboard</strong></span>
      </v-app-bar-title>
      </v-row>
    </button>
    <v-spacer/>

    <v-chip class="theme-button ml-4 mr-9" @click="changeTheme" color="red-darken-5">
      <v-icon class="mr-1" :color="theme.global.name.value === 'light' ? 'white' : 'black'" icon="$whiteBalanceSunny"
              size="18"/>
      <v-icon class="ml-1" :color="theme.global.name.value === 'dark' ? 'white' : 'black'" icon="$weatherNight"
              size="18"/>
    </v-chip>
  </v-app-bar>
</template>

<script setup lang="ts">
import {computed, onBeforeMount, ref} from 'vue'
import { useAppStore } from '@/store/app'
import { useRoute, useRouter } from 'vue-router'
import {useTheme} from 'vuetify'

const route = useRoute()
const router = useRouter()
const theme = useTheme()

const currentRoute = computed(() => {
  return route.name;
})

const changeTheme = () => {
  if (theme.global.name.value === 'light') {
    theme.global.name.value = 'dark';
    localStorage.setItem('theme', theme.global.name.value)
  } else {
    theme.global.name.value = 'light'
    localStorage.setItem('theme', theme.global.name.value)
  }
}

const navigateHome = () => {
  router.push({name: 'home'})
}


onBeforeMount(() => {
  if (localStorage.getItem('theme')){
    theme.global.name.value = localStorage.getItem('theme')
  } else {
    theme.global.name.value = 'light'
    localStorage.setItem('theme', 'light')
  }
})
</script>