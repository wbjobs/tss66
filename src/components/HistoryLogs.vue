<template>
  <div class="history-panel">
    <div class="panel-header">
      <h2>历史日志</h2>
      <div class="header-controls">
        <select v-model="activeTab" class="tab-select">
          <option value="events">事件日志</option>
          <option value="metrics">CPU/内存记录</option>
        </select>
        <select v-if="activeTab === 'metrics'" v-model="filterProcess" class="filter-select">
          <option value="">全部进程</option>
          <option v-for="p in props.processes" :key="p.name" :value="p.name">{{ p.name }}</option>
        </select>
        <select v-if="activeTab === 'events'" v-model="filterEvent" class="filter-select">
          <option value="">全部事件</option>
          <option value="PROCESS_KILLED">进程终止</option>
          <option value="PROCESS_LAUNCHED">进程启动</option>
        </select>
        <select v-model="logLimit" class="filter-select">
          <option :value="50">最近50条</option>
          <option :value="200">最近200条</option>
          <option :value="500">最近500条</option>
        </select>
        <button class="btn-refresh" @click="refresh" title="刷新">🔄</button>
        <button class="btn-cleanup" @click="handleCleanup" title="清理30天前日志">🗑️ 清理旧日志</button>
      </div>
    </div>

    <div v-if="activeTab === 'events'" class="logs-section">
      <table v-if="eventLogs.length > 0" class="log-table">
        <thead>
          <tr>
            <th style="width: 170px">时间</th>
            <th style="width: 100px">事件</th>
            <th style="width: 160px">进程</th>
            <th>详情</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="log in eventLogs" :key="log.id">
            <td class="mono">{{ log.timestamp }}</td>
            <td>
              <span :class="['event-tag', eventClass(log.event_type)]">
                {{ eventLabel(log.event_type) }}
              </span>
            </td>
            <td class="mono">{{ log.process_name }}</td>
            <td>{{ log.detail }}</td>
          </tr>
        </tbody>
      </table>
      <div v-else class="empty-tip">暂无事件记录</div>
    </div>

    <div v-if="activeTab === 'metrics'" class="logs-section">
      <table v-if="metricLogs.length > 0" class="log-table">
        <thead>
          <tr>
            <th style="width: 170px">时间</th>
            <th style="width: 160px">进程</th>
            <th style="width: 90px">状态</th>
            <th style="width: 100px">PID</th>
            <th style="width: 120px">CPU</th>
            <th style="width: 140px">内存</th>
            <th>内存阈值</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="log in metricLogs" :key="log.id"
              :class="{ 'row-stopped': !log.running }">
            <td class="mono">{{ log.timestamp }}</td>
            <td class="mono">{{ log.process_name }}</td>
            <td>
              <span :class="['status-tag', log.running ? 'status-run' : 'status-stop']">
                {{ log.running ? '运行中' : '未运行' }}
              </span>
            </td>
            <td class="mono">{{ log.pid || '-' }}</td>
            <td>
              <div class="mini-cpu-bar">
                <div class="mini-cpu-fill" :style="{ width: Math.min(log.cpu_usage, 100) + '%' }"></div>
                <span>{{ log.cpu_usage.toFixed(1) }}%</span>
              </div>
            </td>
            <td class="mono">{{ formatMb(log.memory_mb) }}</td>
            <td class="mono">{{ log.threshold_mb ? log.threshold_mb.toFixed(0) + ' MB' : '-' }}</td>
          </tr>
        </tbody>
      </table>
      <div v-else class="empty-tip">暂无性能记录</div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

const props = defineProps({
  processes: {
    type: Array,
    required: true
  }
})

const activeTab = ref('events')
const filterProcess = ref('')
const filterEvent = ref('')
const logLimit = ref(200)
const eventLogs = ref([])
const metricLogs = ref([])
let refreshInterval = null

async function refresh() {
  try {
    if (activeTab.value === 'events') {
      eventLogs.value = await invoke('get_event_logs', {
        eventType: filterEvent.value || null,
        limit: logLimit.value
      })
    } else {
      metricLogs.value = await invoke('get_metric_logs', {
        processName: filterProcess.value || null,
        limit: logLimit.value
      })
    }
  } catch (e) {
    console.error('获取日志失败:', e)
  }
}

function handleCleanup() {
  if (confirm('确定要清理 30 天之前的所有日志吗？')) {
    invoke('cleanup_logs', { daysKeep: 30 })
      .then(() => {
        alert('清理完成')
        refresh()
      })
      .catch(e => alert('清理失败: ' + e))
  }
}

function eventLabel(t) {
  switch (t) {
    case 'PROCESS_KILLED': return '进程终止'
    case 'PROCESS_LAUNCHED': return '进程启动'
    default: return t
  }
}

function eventClass(t) {
  switch (t) {
    case 'PROCESS_KILLED': return 'evt-killed'
    case 'PROCESS_LAUNCHED': return 'evt-launched'
    default: return 'evt-default'
  }
}

function formatMb(mb) {
  if (mb >= 1024) return (mb / 1024).toFixed(2) + ' GB'
  return mb.toFixed(1) + ' MB'
}

onMounted(() => {
  refresh()
  refreshInterval = setInterval(refresh, 6000)
})

onUnmounted(() => {
  if (refreshInterval) clearInterval(refreshInterval)
})
</script>

<style scoped>
.history-panel {
  background: var(--bg-secondary);
  border-radius: 12px;
  padding: 20px;
  box-shadow: var(--shadow);
  transition: background 0.3s ease;
  max-height: 75vh;
  display: flex;
  flex-direction: column;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  gap: 10px;
  flex-wrap: wrap;
}

h2 {
  font-size: 18px;
  color: var(--text-primary);
}

.header-controls {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.tab-select,
.filter-select {
  padding: 6px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 13px;
  background: var(--bg-tertiary);
  color: var(--text-primary);
  cursor: pointer;
}

.btn-refresh {
  width: 34px;
  height: 34px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-tertiary);
  cursor: pointer;
  font-size: 15px;
  transition: transform 0.2s;
}

.btn-refresh:hover {
  transform: rotate(360deg);
}

.btn-cleanup {
  padding: 6px 14px;
  border: 1px solid var(--danger-color);
  color: var(--danger-color);
  background: transparent;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  transition: all 0.2s;
}

.btn-cleanup:hover {
  background: var(--danger-color);
  color: white;
}

.logs-section {
  flex: 1;
  overflow: auto;
  border: 1px solid var(--border-light);
  border-radius: 8px;
}

.log-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.log-table thead th {
  position: sticky;
  top: 0;
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  font-weight: 600;
  padding: 10px 12px;
  text-align: left;
  border-bottom: 2px solid var(--border-light);
  z-index: 1;
}

.log-table tbody td {
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-light);
  color: var(--text-primary);
  vertical-align: middle;
}

.log-table tbody tr {
  transition: background 0.15s;
}

.log-table tbody tr:hover {
  background: var(--bg-tertiary);
}

.log-table tbody tr.row-stopped {
  opacity: 0.7;
  background: var(--row-stopped-bg);
}

.mono {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

.event-tag {
  display: inline-block;
  padding: 2px 10px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 500;
}

.evt-killed {
  background: rgba(255, 71, 87, 0.15);
  color: var(--danger-color);
}

.evt-launched {
  background: rgba(46, 213, 115, 0.15);
  color: var(--success-color);
}

.evt-default {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}

.status-tag {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 500;
}

.status-run {
  background: rgba(46, 213, 115, 0.15);
  color: var(--success-color);
}

.status-stop {
  background: rgba(255, 71, 87, 0.15);
  color: var(--danger-color);
}

.mini-cpu-bar {
  position: relative;
  height: 18px;
  background: var(--bg-tertiary);
  border-radius: 9px;
  overflow: hidden;
}

.mini-cpu-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--primary-color), var(--header-gradient-end));
  border-radius: 9px;
  transition: width 0.3s;
}

.mini-cpu-bar span {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 11px;
  font-weight: 600;
  color: var(--text-primary);
  text-shadow: 0 0 2px var(--bg-secondary);
}

.empty-tip {
  padding: 60px 20px;
  text-align: center;
  color: var(--text-muted);
  font-size: 14px;
}
</style>
