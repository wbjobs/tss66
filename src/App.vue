<template>
  <div class="app-container">
    <header class="app-header">
      <h1>进程守护监控</h1>
      <div class="header-actions">
      </div>
    </header>

    <main class="app-main">
      <ConfigPanel
      :config="config"
      @update-config="updateConfig"
      @add-process="addProcess"
      @remove-process="removeProcess"
      @update-threshold="updateThreshold"
      />

      <ProcessTable
      :processes="processData"
      :guardianStatus="guardianStatus"
      />

      <ProcessChart
      :processes="processData"
      :historyData="historyData"
      />
    </main>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import ConfigPanel from './components/ConfigPanel.vue'
import ProcessTable from './components/ProcessTable.vue'
import ProcessChart from './components/ProcessChart.vue'

const config = ref({
  process_names: [],
  thresholds: {}
})

const processData = ref([])
const historyData = ref({})
const guardianStatus = ref({})
let unlistenGuardian = null

async function loadConfig() {
  try {
    const cfg = await invoke('get_config')
    config.value = cfg
  } catch (e) {
    console.error('加载配置失败:', e)
  }
}

async function updateProcessData() {
  try {
    const data = await invoke('get_process_data')
    processData.value = data
  } catch (e) {
    console.error('获取进程数据失败:', e)
  }
}

async function updateHistoryData() {
  try {
    const data = await invoke('get_history_data')
    historyData.value = data
  } catch (e) {
    console.error('获取历史数据失败:', e)
  }
}

async function loadGuardianStatus() {
  try {
    const status = await invoke('get_guardian_status')
    guardianStatus.value = status
  } catch (e) {
    console.error('获取守护状态失败:', e)
  }
}

async function updateConfig(newConfig) {
  try {
    await invoke('update_config', { config: newConfig })
    config.value = newConfig
  } catch (e) {
    console.error('更新配置失败:', e)
  }
}

async function addProcess(processName) {
  try {
    await invoke('add_process', { processName })
    await loadConfig()
  } catch (e) {
    console.error('添加进程失败:', e)
  }
}

async function removeProcess(processName) {
  try {
    await invoke('remove_process', { processName })
    await loadConfig()
  } catch (e) {
    console.error('移除进程失败:', e)
  }
}

async function updateThreshold(processName, thresholdMb) {
  try {
    await invoke('set_memory_threshold', { processName, thresholdMb })
    await loadConfig()
  } catch (e) {
    console.error('设置阈值失败:', e)
  }
}

let fastPollInterval = null
let slowPollInterval = null

onMounted(async () => {
  await loadConfig()
  await updateProcessData()
  await updateHistoryData()
  await loadGuardianStatus()

  fastPollInterval = setInterval(async () => {
    await updateProcessData()
    await loadGuardianStatus()
  }, 2000)

  slowPollInterval = setInterval(async () => {
    await updateHistoryData()
  }, 6000)

  unlistenGuardian = await listen('process-killed', (event) => {
    console.log('进程被终止:', event.payload)
    alert(`进程 ${event.payload.process_name} 因内存超限被终止`)
  })
})

onUnmounted(() => {
  if (fastPollInterval) {
    clearInterval(fastPollInterval)
  }
  if (slowPollInterval) {
    clearInterval(slowPollInterval)
  }
  if (unlistenGuardian) {
    unlistenGuardian()
  }
})
</script>

<style scoped>
.app-container {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.app-header {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 20px 30px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: 0 2px 10px rgba(0,0,0,0.1);
}

.app-header h1 {
  font-size: 24px;
  font-weight: 600;
}

.app-main {
  flex: 1;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}
</style>
