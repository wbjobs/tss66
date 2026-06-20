<template>
  <div class="app-container">
    <header class="app-header">
      <h1>进程守护监控</h1>
      <div class="header-actions">
        <button
          class="theme-toggle"
          :title="isDark ? '切换到亮色模式' : '切换到暗色模式'"
          @click="toggleTheme"
        >
          {{ isDark ? '☀️' : '🌙' }}
        </button>
      </div>
    </header>

    <nav class="app-tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="['tab-btn', { active: activeTab === tab.id }]"
        @click="activeTab = tab.id"
      >
        <span class="tab-icon">{{ tab.icon }}</span>
        <span class="tab-label">{{ tab.label }}</span>
      </button>
    </nav>

    <main class="app-main">
      <div v-show="activeTab === 'monitor'">
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
      </div>

      <div v-show="activeTab === 'launcher'">
        <ProcessLauncher
          :config="config"
          @add-entry="addLauncherEntry"
          @remove-entry="removeLauncherEntry"
          @launch="launchProgram"
          @reload="loadConfig"
        />
      </div>

      <div v-show="activeTab === 'history'">
        <HistoryLogs
          :processes="processData"
        />
      </div>
    </main>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import ConfigPanel from './components/ConfigPanel.vue'
import ProcessTable from './components/ProcessTable.vue'
import ProcessChart from './components/ProcessChart.vue'
import ProcessLauncher from './components/ProcessLauncher.vue'
import HistoryLogs from './components/HistoryLogs.vue'

const tabs = [
  { id: 'monitor', label: '实时监控', icon: '📊' },
  { id: 'launcher', label: '进程启动器', icon: '🚀' },
  { id: 'history', label: '历史日志', icon: '📜' },
]
const activeTab = ref('monitor')

const isDark = ref(false)

function toggleTheme() {
  isDark.value = !isDark.value
  const root = document.documentElement
  if (isDark.value) {
    root.classList.add('dark')
  } else {
    root.classList.remove('dark')
  }
  try {
    localStorage.setItem('theme', isDark.value ? 'dark' : 'light')
  } catch (e) {}
}

function initTheme() {
  let stored = null
  try {
    stored = localStorage.getItem('theme')
  } catch (e) {}
  if (stored === 'dark') {
    isDark.value = true
  } else if (stored === 'light') {
    isDark.value = false
  } else {
    isDark.value = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches
  }
  const root = document.documentElement
  if (isDark.value) {
    root.classList.add('dark')
  }
}

const config = ref({
  process_names: [],
  thresholds: {},
  launcher: []
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

// ============ Launcher ============

async function addLauncherEntry(entry) {
  try {
    await invoke('add_launcher_entry', { entry })
    await loadConfig()
  } catch (e) {
    console.error('添加启动项失败:', e)
    alert('添加启动项失败: ' + e)
  }
}

async function removeLauncherEntry(label) {
  try {
    await invoke('remove_launcher_entry', { label })
    await loadConfig()
  } catch (e) {
    console.error('删除启动项失败:', e)
    alert('删除启动项失败: ' + e)
  }
}

async function launchProgram(label) {
  try {
    const result = await invoke('launch_program', { label })
    await loadConfig()
    console.log(result)
  } catch (e) {
    console.error('启动程序失败:', e)
    alert('启动失败: ' + e)
  }
}

let fastPollInterval = null
let slowPollInterval = null

onMounted(async () => {
  initTheme()
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
  background: var(--bg-primary);
  transition: background 0.3s ease;
}

.app-header {
  background: linear-gradient(135deg, var(--header-gradient-start) 0%, var(--header-gradient-end) 100%);
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

.theme-toggle {
  background: rgba(255, 255, 255, 0.2);
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: white;
  font-size: 20px;
  padding: 8px 14px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.theme-toggle:hover {
  background: rgba(255, 255, 255, 0.3);
  transform: scale(1.05);
}

.app-tabs {
  display: flex;
  gap: 4px;
  padding: 12px 30px 0;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-light);
  transition: background 0.3s ease;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 12px 20px;
  background: transparent;
  border: none;
  border-bottom: 3px solid transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s ease;
  border-radius: 8px 8px 0 0;
}

.tab-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.tab-btn.active {
  background: var(--bg-primary);
  color: var(--primary-color);
  border-bottom-color: var(--primary-color);
}

.tab-icon {
  font-size: 16px;
}

.app-main {
  flex: 1;
  padding: 20px 30px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  background: var(--bg-primary);
  transition: background 0.3s ease;
}
</style>
