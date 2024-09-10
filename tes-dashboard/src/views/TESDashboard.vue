<template>
  <div class="w-100 px-12 mt-5" style="border-radius: 10px">
      <div v-if="tasks.length > 0">
        <v-row class="pb-12">
          <v-col cols="12" md="4" class="d-flex justify-center">
            <Chart
                :data="chartData"
                :options="chartOptions"
                type="pie"
                style="max-width: 225px"
                v-show="tasks.length > 0"
            />
          </v-col>
          <v-col cols="12" md="2" class="d-flex flex-column justify-center align-center">
            <h3>Tasks Completed</h3>
            <h1 class="count" >{{taskCounts.complete}}</h1>
          </v-col>
          <v-col cols="12" md="2" class="d-flex flex-column justify-center align-center">
            <h3>In Progress</h3>
            <h1 class="count">{{ taskCounts.running }}</h1>
          </v-col>
          <v-col cols="12" md="2" class="d-flex flex-column justify-center align-center">
            <h3>Failed</h3>
            <h1 class="count">{{ taskCounts.error}}</h1>
          </v-col>
        </v-row>
      </div>
      <div v-else style="min-height: 300px; display: flex; justify-content: center; align-items: center">
        <v-progress-circular size="200" v-if="tableLoading" indeterminate/>
      </div>
    <v-data-table
        class="mt-n13"
        density="compact"
        :headers="headers"
        :items="tasks"
        :loading="tableLoading"
        style="transform: scale(0.92)"
    />
  </div>
  <Snackbar
      :snack="snackData.on"
      :snackColor="snackData.color"
      :snackText="snackData.text"
      v-model="snackData.on"
  />
</template>
<script setup lang="ts">
import AutoComplete from 'primevue/autocomplete'
import Chart from 'primevue/Chart'
import {ErrorService, SnackBarData} from '@/services/error.service.ts'
import {headers} from '@/app-config/TES-Dashboard.ts'
import {computed, onMounted, onUnmounted, ref, watch} from 'vue'
import {TaskService} from '@/services/task.service.ts'
import Snackbar from "@/components/Snackbar.vue"
import {useTheme} from "vuetify"

const theme = useTheme()
const currentTheme = computed(() => theme.global.name.value)

const chartOptions = ref({
  animation: {
    duration: 0
  }
})

const snackData = ref<SnackBarData>({on: false, color: '', text: ''})
const tableLoading = ref(false);
const taskCounts = ref({
  complete: 0,
  running: 0,
  error: 0
})
const tasks = ref([])

const PENDING_STATES = ["QUEUED", "INITIALIZING", "RUNNING",]
const FINISHED_STATES = ["COMPLETE", "CANCELED"]
const ERROR_STATES = ["EXECUTOR_ERROR", "SYSTEM_ERROR"]

const chartData = ref({
  labels: ['Complete', 'Running', 'Error'],
  datasets: [
    {
      data: [0, 0, 0],
      backgroundColor: ["#7CB342", "#FFCA28", "#EF5350"],
      hoverBackgroundColor: ["#558B2F", "#FFB300", "#E53935"]
    }
  ]
})

// function that calls listTask at a set interval
const listTasksInterval = async () => {
  await listTasks()
  setTimeout(listTasksInterval, 200)
}

const listTasks = async () => {
  try {
    let listTasksResponse = await TaskService.listTasks('tasks');
    listTasksResponse = listTasksResponse.tasks.filter((task: any) => new Date(task.creation_time) > new Date(new Date().getTime() - 1000 * 60 * 60 * 24))
    tasks.value = listTasksResponse.map((task: any) => {
      let start_date = new Date(task.logs[0].start_time)
      let end_date = new Date(task.logs[0].end_time)
      let start_date_time = start_date.toLocaleDateString('en-US') + ' ' + start_date.toLocaleTimeString('en-US')
      let end_date_time = end_date.toLocaleDateString('en-US') + ' ' + end_date.toLocaleTimeString('en-US')
      return {
        id: task.id,
        name: task.name,
        state: task.state,
        description: task.description,
        start_time: start_date_time,
        end_time: end_date_time,
      }
    })
    updateChartData();

  } catch (error) {
    await ErrorService.handleError(`Couldn't get tasks`, snackData.value, error)
  }
}

const updateChartData = () => {
  let completeTaskCount = tasks.value.filter((task: any) => FINISHED_STATES.includes(task.state)).length;
  let runningTaskCount = tasks.value.filter((task: any) => PENDING_STATES.includes(task.state)).length;
  let errorTaskCount = tasks.value.filter((task: any) => ERROR_STATES.includes(task.state)).length;
  chartData.value.datasets[0].data = [
    completeTaskCount,
    runningTaskCount,
    errorTaskCount
  ]
  taskCounts.value.complete = completeTaskCount
  taskCounts.value.running = runningTaskCount
  taskCounts.value.error = errorTaskCount
}

onMounted(async () => {
  tableLoading.value = true;
  await listTasksInterval()
  tableLoading.value = false;
})

onUnmounted(() => {
  // clears the timeout for listTasksInterval
  clearTimeout(listTasksInterval)
})
</script>

<style scoped>
.count {
  font-size: 96px;
}
</style>